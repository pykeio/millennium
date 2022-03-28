// Copyright 2022 pyke.io
//           2019-2021 Tauri Programme within The Commons Conservancy
//                     [https://tauri.studio/]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap, HashSet
	},
	fmt,
	ops::Deref,
	path::PathBuf,
	sync::{
		mpsc::{channel, Sender},
		Arc, Mutex, MutexGuard, Weak
	},
	thread::{current as current_thread, ThreadId}
};

use millennium_runtime::window::MenuEvent;
use millennium_runtime::{
	http::{Request as HttpRequest, RequestParts as HttpRequestParts, Response as HttpResponse, ResponseParts as HttpResponseParts},
	menu::{CustomMenuItem, Menu, MenuEntry, MenuHash, MenuId, MenuItem, MenuUpdate},
	monitor::Monitor,
	webview::{WebviewIpcHandler, WindowBuilder, WindowBuilderBase},
	window::{
		dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
		DetachedWindow, FileDropEvent, JsEventListenerKey, PendingWindow, WindowEvent
	},
	ClipboardManager, Dispatch, Error, EventLoopProxy, ExitRequestedEventAction, GlobalShortcutManager, Result, RunEvent, RunIteration, Runtime, RuntimeHandle,
	UserAttentionType, UserEvent, WindowIcon
};
#[cfg(target_os = "macos")]
use millennium_runtime::{menu::NativeImage, ActivationPolicy};
#[cfg(feature = "system-tray")]
use millennium_runtime::{SystemTray, SystemTrayEvent};
use millennium_utils::config::WindowConfig;
#[cfg(target_os = "macos")]
pub use millennium_webview::application::platform::macos::{
	ActivationPolicy as MillenniumActivationPolicy, CustomMenuItemExtMacOS, EventLoopExtMacOS, NativeImage as MillenniumNativeImage, WindowExtMacOS
};
#[cfg(all(feature = "system-tray", target_os = "macos"))]
use millennium_webview::application::platform::macos::{SystemTrayBuilderExtMacOS, SystemTrayExtMacOS};
#[cfg(target_os = "linux")]
use millennium_webview::application::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};
#[cfg(windows)]
use millennium_webview::application::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
#[cfg(feature = "system-tray")]
use millennium_webview::application::system_tray::{SystemTray as MillenniumSystemTray, SystemTrayBuilder};
pub use millennium_webview::application::window::{Window, WindowBuilder as MillenniumWindowBuilder, WindowId};
#[cfg(windows)]
#[allow(unused)]
use millennium_webview::webview::WebviewExtWindows;
use millennium_webview::{
	application::{
		accelerator::{Accelerator, AcceleratorId},
		clipboard::Clipboard,
		dpi::{
			LogicalPosition as MillenniumLogicalPosition, LogicalSize as MillenniumLogicalSize, PhysicalPosition as MillenniumPhysicalPosition,
			PhysicalSize as MillenniumPhysicalSize, Position as MillenniumPosition, Size as MillenniumSize
		},
		event::{Event, StartCause, WindowEvent as MillenniumWindowEvent},
		event_loop::{ControlFlow, EventLoop, EventLoopProxy as MillenniumEventLoopProxy, EventLoopWindowTarget},
		global_shortcut::{GlobalShortcut, ShortcutManager as MillenniumShortcutManager},
		menu::{
			CustomMenuItem as MillenniumCustomMenuItem, MenuBar, MenuId as MillenniumMenuId, MenuItem as MillenniumMenuItem,
			MenuItemAttributes as MillenniumMenuItemAttributes, MenuType
		},
		monitor::MonitorHandle,
		window::{Fullscreen, Icon as MillenniumWindowIcon, UserAttentionType as MillenniumUserAttentionType}
	},
	http::{
		Request as MillenniumHttpRequest, RequestParts as MillenniumRequestParts, Response as MillenniumHttpResponse, ResponseParts as MillenniumResponseParts
	},
	webview::{FileDropEvent as MillenniumFileDropEvent, WebContext, WebView, WebViewBuilder}
};
use uuid::Uuid;
#[cfg(all(windows, not(feature = "__rust_analyzer_hack")))]
use webview2_com::FocusChangedEventHandler;
#[cfg(windows)]
#[allow(unused_imports)]
use windows::Win32::{Foundation::HWND, System::WinRT::EventRegistrationToken};

type WebviewId = u64;

#[cfg(feature = "system-tray")]
mod system_tray;
#[cfg(feature = "system-tray")]
use system_tray::*;

type WebContextStore = Arc<Mutex<HashMap<Option<PathBuf>, WebContext>>>;
// window
type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
type WindowEventListenersMap = Arc<Mutex<HashMap<Uuid, WindowEventHandler>>>;
type WindowEventListeners = Arc<Mutex<HashMap<WebviewId, WindowEventListenersMap>>>;
// global shortcut
type GlobalShortcutListeners = Arc<Mutex<HashMap<AcceleratorId, Box<dyn Fn() + Send>>>>;
// menu
pub type MenuEventHandler = Box<dyn Fn(&MenuEvent) + Send>;
pub type MenuEventListeners = Arc<Mutex<HashMap<WebviewId, WindowMenuEventListeners>>>;
pub type WindowMenuEventListeners = Arc<Mutex<HashMap<Uuid, MenuEventHandler>>>;

#[derive(Debug, Clone, Default)]
struct WebviewIdStore(Arc<Mutex<HashMap<WindowId, WebviewId>>>);

impl WebviewIdStore {
	fn insert(&self, w: WindowId, id: WebviewId) {
		self.0.lock().unwrap().insert(w, id);
	}

	fn get(&self, w: &WindowId) -> WebviewId {
		*self.0.lock().unwrap().get(w).unwrap()
	}
}

macro_rules! getter {
	($self: ident, $rx: expr, $message: expr) => {{
		send_user_message(&$self.context, $message)?;
		$rx.recv().map_err(|_| Error::FailedToReceiveMessage)
	}};
}

macro_rules! window_getter {
	($self: ident, $message: expr) => {{
		let (tx, rx) = channel();
		getter!($self, rx, Message::Window($self.window_id, $message(tx)))
	}};
}

fn send_user_message<T: UserEvent>(context: &Context<T>, message: Message<T>) -> Result<()> {
	if current_thread().id() == context.main_thread_id {
		handle_user_message(
			&context.main_thread.window_target,
			message,
			UserMessageContext {
				window_event_listeners: &context.window_event_listeners,
				global_shortcut_manager: context.main_thread.global_shortcut_manager.clone(),
				clipboard_manager: context.main_thread.clipboard_manager.clone(),
				menu_event_listeners: &context.menu_event_listeners,
				windows: context.main_thread.windows.clone(),
				#[cfg(feature = "system-tray")]
				tray_context: &context.main_thread.tray_context
			},
			&context.main_thread.web_context
		);
		Ok(())
	} else {
		context.proxy.send_event(message).map_err(|_| Error::FailedToSendMessage)
	}
}

#[derive(Clone)]
struct Context<T: UserEvent> {
	webview_id_map: WebviewIdStore,
	main_thread_id: ThreadId,
	proxy: MillenniumEventLoopProxy<Message<T>>,
	window_event_listeners: WindowEventListeners,
	menu_event_listeners: MenuEventListeners,
	main_thread: DispatcherMainThreadContext<T>
}

impl<T: UserEvent> Context<T> {
	fn prepare_window(&self, window_id: WebviewId) {
		self.window_event_listeners
			.lock()
			.unwrap()
			.insert(window_id, WindowEventListenersMap::default());
		self.menu_event_listeners
			.lock()
			.unwrap()
			.insert(window_id, WindowMenuEventListeners::default());
	}

	fn create_webview(&self, pending: PendingWindow<T, MillenniumWebview<T>>) -> Result<DetachedWindow<T, MillenniumWebview<T>>> {
		let label = pending.label.clone();
		let menu_ids = pending.menu_ids.clone();
		let js_event_listeners = pending.js_event_listeners.clone();
		let context = self.clone();
		let window_id = rand::random();

		self.prepare_window(window_id);

		self.proxy
			.send_event(Message::CreateWebview(
				window_id,
				Box::new(move |event_loop, web_context| create_webview(window_id, event_loop, web_context, context, pending))
			))
			.map_err(|_| Error::FailedToSendMessage)?;

		let dispatcher = MillenniumDispatcher { window_id, context: self.clone() };
		Ok(DetachedWindow {
			label,
			dispatcher,
			menu_ids,
			js_event_listeners
		})
	}
}

#[derive(Debug, Clone)]
struct DispatcherMainThreadContext<T: UserEvent> {
	window_target: EventLoopWindowTarget<Message<T>>,
	web_context: WebContextStore,
	global_shortcut_manager: Arc<Mutex<MillenniumShortcutManager>>,
	clipboard_manager: Arc<Mutex<Clipboard>>,
	windows: Arc<Mutex<HashMap<WebviewId, WindowWrapper>>>,
	#[cfg(feature = "system-tray")]
	tray_context: TrayContext
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Send for DispatcherMainThreadContext<T> {}

impl<T: UserEvent> fmt::Debug for Context<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Context")
			.field("main_thread_id", &self.main_thread_id)
			.field("proxy", &self.proxy)
			.field("main_thread", &self.main_thread)
			.finish()
	}
}

struct HttpRequestPartsWrapper(HttpRequestParts);

impl From<HttpRequestPartsWrapper> for HttpRequestParts {
	fn from(parts: HttpRequestPartsWrapper) -> Self {
		Self {
			method: parts.0.method,
			uri: parts.0.uri,
			headers: parts.0.headers
		}
	}
}

impl From<HttpRequestParts> for HttpRequestPartsWrapper {
	fn from(request: HttpRequestParts) -> Self {
		Self(HttpRequestParts {
			method: request.method,
			uri: request.uri,
			headers: request.headers
		})
	}
}

impl From<MillenniumRequestParts> for HttpRequestPartsWrapper {
	fn from(request: MillenniumRequestParts) -> Self {
		Self(HttpRequestParts {
			method: request.method,
			uri: request.uri,
			headers: request.headers
		})
	}
}

struct HttpRequestWrapper(HttpRequest);

impl From<&MillenniumHttpRequest> for HttpRequestWrapper {
	fn from(req: &MillenniumHttpRequest) -> Self {
		Self(HttpRequest::new_internal(HttpRequestPartsWrapper::from(req.head.clone()).0, req.body.clone()))
	}
}

// response
struct HttpResponsePartsWrapper(MillenniumResponseParts);
impl From<HttpResponseParts> for HttpResponsePartsWrapper {
	fn from(response: HttpResponseParts) -> Self {
		Self(MillenniumResponseParts {
			mimetype: response.mimetype,
			status: response.status,
			version: response.version,
			headers: response.headers
		})
	}
}

struct HttpResponseWrapper(MillenniumHttpResponse);
impl From<HttpResponse> for HttpResponseWrapper {
	fn from(response: HttpResponse) -> Self {
		let (parts, body) = response.into_parts();
		Self(MillenniumHttpResponse {
			body,
			head: HttpResponsePartsWrapper::from(parts).0
		})
	}
}

pub struct MenuItemAttributesWrapper<'a>(pub MillenniumMenuItemAttributes<'a>);

impl<'a> From<&'a CustomMenuItem> for MenuItemAttributesWrapper<'a> {
	fn from(item: &'a CustomMenuItem) -> Self {
		let mut attributes = MillenniumMenuItemAttributes::new(&item.title)
			.with_enabled(item.enabled)
			.with_selected(item.selected)
			.with_id(MillenniumMenuId(item.id));
		if let Some(accelerator) = item.keyboard_accelerator.as_ref() {
			attributes = attributes.with_accelerators(&accelerator.parse().expect("invalid accelerator"));
		}
		Self(attributes)
	}
}

pub struct MenuItemWrapper(pub MillenniumMenuItem);

impl From<MenuItem> for MenuItemWrapper {
	fn from(item: MenuItem) -> Self {
		match item {
			MenuItem::About(v) => Self(MillenniumMenuItem::About(v)),
			MenuItem::Hide => Self(MillenniumMenuItem::Hide),
			MenuItem::Services => Self(MillenniumMenuItem::Services),
			MenuItem::HideOthers => Self(MillenniumMenuItem::HideOthers),
			MenuItem::ShowAll => Self(MillenniumMenuItem::ShowAll),
			MenuItem::CloseWindow => Self(MillenniumMenuItem::CloseWindow),
			MenuItem::Quit => Self(MillenniumMenuItem::Quit),
			MenuItem::Copy => Self(MillenniumMenuItem::Copy),
			MenuItem::Cut => Self(MillenniumMenuItem::Cut),
			MenuItem::Undo => Self(MillenniumMenuItem::Undo),
			MenuItem::Redo => Self(MillenniumMenuItem::Redo),
			MenuItem::SelectAll => Self(MillenniumMenuItem::SelectAll),
			MenuItem::Paste => Self(MillenniumMenuItem::Paste),
			MenuItem::EnterFullScreen => Self(MillenniumMenuItem::EnterFullScreen),
			MenuItem::Minimize => Self(MillenniumMenuItem::Minimize),
			MenuItem::Zoom => Self(MillenniumMenuItem::Zoom),
			MenuItem::Separator => Self(MillenniumMenuItem::Separator),
			_ => unimplemented!()
		}
	}
}

#[cfg(target_os = "macos")]
pub struct NativeImageWrapper(pub MillenniumNativeImage);

#[cfg(target_os = "macos")]
impl std::fmt::Debug for NativeImageWrapper {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("NativeImageWrapper").finish()
	}
}

#[cfg(target_os = "macos")]
impl From<NativeImage> for NativeImageWrapper {
	fn from(image: NativeImage) -> NativeImageWrapper {
		let millennium_image = match image {
			NativeImage::Add => MillenniumNativeImage::Add,
			NativeImage::Advanced => MillenniumNativeImage::Advanced,
			NativeImage::Bluetooth => MillenniumNativeImage::Bluetooth,
			NativeImage::Bookmarks => MillenniumNativeImage::Bookmarks,
			NativeImage::Caution => MillenniumNativeImage::Caution,
			NativeImage::ColorPanel => MillenniumNativeImage::ColorPanel,
			NativeImage::ColumnView => MillenniumNativeImage::ColumnView,
			NativeImage::Computer => MillenniumNativeImage::Computer,
			NativeImage::EnterFullScreen => MillenniumNativeImage::EnterFullScreen,
			NativeImage::Everyone => MillenniumNativeImage::Everyone,
			NativeImage::ExitFullScreen => MillenniumNativeImage::ExitFullScreen,
			NativeImage::FlowView => MillenniumNativeImage::FlowView,
			NativeImage::Folder => MillenniumNativeImage::Folder,
			NativeImage::FolderBurnable => MillenniumNativeImage::FolderBurnable,
			NativeImage::FolderSmart => MillenniumNativeImage::FolderSmart,
			NativeImage::FollowLinkFreestanding => MillenniumNativeImage::FollowLinkFreestanding,
			NativeImage::FontPanel => MillenniumNativeImage::FontPanel,
			NativeImage::GoLeft => MillenniumNativeImage::GoLeft,
			NativeImage::GoRight => MillenniumNativeImage::GoRight,
			NativeImage::Home => MillenniumNativeImage::Home,
			NativeImage::IChatTheater => MillenniumNativeImage::IChatTheater,
			NativeImage::IconView => MillenniumNativeImage::IconView,
			NativeImage::Info => MillenniumNativeImage::Info,
			NativeImage::InvalidDataFreestanding => MillenniumNativeImage::InvalidDataFreestanding,
			NativeImage::LeftFacingTriangle => MillenniumNativeImage::LeftFacingTriangle,
			NativeImage::ListView => MillenniumNativeImage::ListView,
			NativeImage::LockLocked => MillenniumNativeImage::LockLocked,
			NativeImage::LockUnlocked => MillenniumNativeImage::LockUnlocked,
			NativeImage::MenuMixedState => MillenniumNativeImage::MenuMixedState,
			NativeImage::MenuOnState => MillenniumNativeImage::MenuOnState,
			NativeImage::MobileMe => MillenniumNativeImage::MobileMe,
			NativeImage::MultipleDocuments => MillenniumNativeImage::MultipleDocuments,
			NativeImage::Network => MillenniumNativeImage::Network,
			NativeImage::Path => MillenniumNativeImage::Path,
			NativeImage::PreferencesGeneral => MillenniumNativeImage::PreferencesGeneral,
			NativeImage::QuickLook => MillenniumNativeImage::QuickLook,
			NativeImage::RefreshFreestanding => MillenniumNativeImage::RefreshFreestanding,
			NativeImage::Refresh => MillenniumNativeImage::Refresh,
			NativeImage::Remove => MillenniumNativeImage::Remove,
			NativeImage::RevealFreestanding => MillenniumNativeImage::RevealFreestanding,
			NativeImage::RightFacingTriangle => MillenniumNativeImage::RightFacingTriangle,
			NativeImage::Share => MillenniumNativeImage::Share,
			NativeImage::Slideshow => MillenniumNativeImage::Slideshow,
			NativeImage::SmartBadge => MillenniumNativeImage::SmartBadge,
			NativeImage::StatusAvailable => MillenniumNativeImage::StatusAvailable,
			NativeImage::StatusNone => MillenniumNativeImage::StatusNone,
			NativeImage::StatusPartiallyAvailable => MillenniumNativeImage::StatusPartiallyAvailable,
			NativeImage::StatusUnavailable => MillenniumNativeImage::StatusUnavailable,
			NativeImage::StopProgressFreestanding => MillenniumNativeImage::StopProgressFreestanding,
			NativeImage::StopProgress => MillenniumNativeImage::StopProgress,

			NativeImage::TrashEmpty => MillenniumNativeImage::TrashEmpty,
			NativeImage::TrashFull => MillenniumNativeImage::TrashFull,
			NativeImage::User => MillenniumNativeImage::User,
			NativeImage::UserAccounts => MillenniumNativeImage::UserAccounts,
			NativeImage::UserGroup => MillenniumNativeImage::UserGroup,
			NativeImage::UserGuest => MillenniumNativeImage::UserGuest
		};
		Self(millennium_image)
	}
}

#[derive(Debug, Clone)]
pub struct GlobalShortcutWrapper(GlobalShortcut);

// SAFETY: usage outside of main thread is guarded, we use the event loop on
// such cases.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GlobalShortcutWrapper {}

/// Wrapper around [`MillenniumShortcutManager`].
#[derive(Clone)]
pub struct GlobalShortcutManagerHandle<T: UserEvent> {
	context: Context<T>,
	shortcuts: Arc<Mutex<HashMap<String, (AcceleratorId, GlobalShortcutWrapper)>>>,
	listeners: GlobalShortcutListeners
}

// SAFETY: this is safe since the `Context` usage is guarded on
// `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for GlobalShortcutManagerHandle<T> {}

impl<T: UserEvent> fmt::Debug for GlobalShortcutManagerHandle<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("GlobalShortcutManagerHandle")
			.field("context", &self.context)
			.field("shortcuts", &self.shortcuts)
			.finish()
	}
}

impl<T: UserEvent> GlobalShortcutManager for GlobalShortcutManagerHandle<T> {
	fn is_registered(&self, accelerator: &str) -> Result<bool> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::IsRegistered(accelerator.parse().expect("invalid accelerator"), tx)))
	}

	fn register<F: Fn() + Send + 'static>(&mut self, accelerator: &str, handler: F) -> Result<()> {
		let millennium_accelerator: Accelerator = accelerator.parse().expect("invalid accelerator");
		let id = millennium_accelerator.clone().id();
		let (tx, rx) = channel();
		let shortcut = getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::Register(millennium_accelerator, tx)))??;

		self.listeners.lock().unwrap().insert(id, Box::new(handler));
		self.shortcuts.lock().unwrap().insert(accelerator.into(), (id, shortcut));

		Ok(())
	}

	fn unregister_all(&mut self) -> Result<()> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::UnregisterAll(tx)))??;
		self.listeners.lock().unwrap().clear();
		self.shortcuts.lock().unwrap().clear();
		Ok(())
	}

	fn unregister(&mut self, accelerator: &str) -> Result<()> {
		if let Some((accelerator_id, shortcut)) = self.shortcuts.lock().unwrap().remove(accelerator) {
			let (tx, rx) = channel();
			getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::Unregister(shortcut, tx)))??;
			self.listeners.lock().unwrap().remove(&accelerator_id);
		}
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct ClipboardManagerWrapper<T: UserEvent> {
	context: Context<T>
}

// SAFETY: this is safe since the `Context` usage is guarded on
// `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for ClipboardManagerWrapper<T> {}

impl<T: UserEvent> ClipboardManager for ClipboardManagerWrapper<T> {
	fn read_text(&self) -> Result<Option<String>> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::Clipboard(ClipboardMessage::ReadText(tx)))
	}

	fn write_text<V: Into<String>>(&mut self, text: V) -> Result<()> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::Clipboard(ClipboardMessage::WriteText(text.into(), tx)))?;
		Ok(())
	}
}

/// Wrapper around a [`millennium_webview::application::window::Icon`] that can
/// be created from a [`WindowIcon`].
pub struct MillenniumIcon(MillenniumWindowIcon);

fn icon_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
	Error::InvalidIcon(Box::new(e))
}

impl TryFrom<WindowIcon> for MillenniumIcon {
	type Error = Error;
	fn try_from(icon: WindowIcon) -> std::result::Result<Self, Self::Error> {
		MillenniumWindowIcon::from_rgba(icon.rgba, icon.width, icon.height)
			.map(Self)
			.map_err(icon_err)
	}
}

struct WindowEventWrapper(Option<WindowEvent>);

impl WindowEventWrapper {
	fn parse(webview: &WindowHandle, event: &MillenniumWindowEvent<'_>) -> Self {
		match event {
			// resized event from Millennium Core doesn't include a reliable size on macOS
			// because Millennium Webview replaces the NSView
			MillenniumWindowEvent::Resized(_) => Self(Some(WindowEvent::Resized(PhysicalSizeWrapper(webview.inner_size()).into()))),
			e => e.into()
		}
	}
}

impl<'a> From<&MillenniumWindowEvent<'a>> for WindowEventWrapper {
	fn from(event: &MillenniumWindowEvent<'a>) -> Self {
		let event = match event {
			MillenniumWindowEvent::Resized(size) => WindowEvent::Resized(PhysicalSizeWrapper(*size).into()),
			MillenniumWindowEvent::Moved(position) => WindowEvent::Moved(PhysicalPositionWrapper(*position).into()),
			MillenniumWindowEvent::Destroyed => WindowEvent::Destroyed,
			MillenniumWindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => WindowEvent::ScaleFactorChanged {
				scale_factor: *scale_factor,
				new_inner_size: PhysicalSizeWrapper(**new_inner_size).into()
			},
			#[cfg(any(target_os = "linux", target_os = "macos"))]
			MillenniumWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
			_ => return Self(None)
		};
		Self(Some(event))
	}
}

impl From<&WebviewEvent> for WindowEventWrapper {
	fn from(event: &WebviewEvent) -> Self {
		let event = match event {
			WebviewEvent::Focused(focused) => WindowEvent::Focused(*focused)
		};
		Self(Some(event))
	}
}

pub struct MonitorHandleWrapper(MonitorHandle);

impl From<MonitorHandleWrapper> for Monitor {
	fn from(monitor: MonitorHandleWrapper) -> Monitor {
		Self {
			name: monitor.0.name(),
			position: PhysicalPositionWrapper(monitor.0.position()).into(),
			size: PhysicalSizeWrapper(monitor.0.size()).into(),
			scale_factor: monitor.0.scale_factor()
		}
	}
}

struct PhysicalPositionWrapper<T>(MillenniumPhysicalPosition<T>);

impl<T> From<PhysicalPositionWrapper<T>> for PhysicalPosition<T> {
	fn from(position: PhysicalPositionWrapper<T>) -> Self {
		Self { x: position.0.x, y: position.0.y }
	}
}

impl<T> From<PhysicalPosition<T>> for PhysicalPositionWrapper<T> {
	fn from(position: PhysicalPosition<T>) -> Self {
		Self(MillenniumPhysicalPosition { x: position.x, y: position.y })
	}
}

struct LogicalPositionWrapper<T>(MillenniumLogicalPosition<T>);

impl<T> From<LogicalPosition<T>> for LogicalPositionWrapper<T> {
	fn from(position: LogicalPosition<T>) -> Self {
		Self(MillenniumLogicalPosition { x: position.x, y: position.y })
	}
}

struct PhysicalSizeWrapper<T>(MillenniumPhysicalSize<T>);

impl<T> From<PhysicalSizeWrapper<T>> for PhysicalSize<T> {
	fn from(size: PhysicalSizeWrapper<T>) -> Self {
		Self {
			width: size.0.width,
			height: size.0.height
		}
	}
}

impl<T> From<PhysicalSize<T>> for PhysicalSizeWrapper<T> {
	fn from(size: PhysicalSize<T>) -> Self {
		Self(MillenniumPhysicalSize {
			width: size.width,
			height: size.height
		})
	}
}

struct LogicalSizeWrapper<T>(MillenniumLogicalSize<T>);

impl<T> From<LogicalSize<T>> for LogicalSizeWrapper<T> {
	fn from(size: LogicalSize<T>) -> Self {
		Self(MillenniumLogicalSize {
			width: size.width,
			height: size.height
		})
	}
}

struct SizeWrapper(MillenniumSize);

impl From<Size> for SizeWrapper {
	fn from(size: Size) -> Self {
		match size {
			Size::Logical(s) => Self(MillenniumSize::Logical(LogicalSizeWrapper::from(s).0)),
			Size::Physical(s) => Self(MillenniumSize::Physical(PhysicalSizeWrapper::from(s).0))
		}
	}
}

struct PositionWrapper(MillenniumPosition);

impl From<Position> for PositionWrapper {
	fn from(position: Position) -> Self {
		match position {
			Position::Logical(s) => Self(MillenniumPosition::Logical(LogicalPositionWrapper::from(s).0)),
			Position::Physical(s) => Self(MillenniumPosition::Physical(PhysicalPositionWrapper::from(s).0))
		}
	}
}

#[derive(Debug, Clone)]
pub struct UserAttentionTypeWrapper(MillenniumUserAttentionType);

impl From<UserAttentionType> for UserAttentionTypeWrapper {
	fn from(request_type: UserAttentionType) -> UserAttentionTypeWrapper {
		let o = match request_type {
			UserAttentionType::Critical => MillenniumUserAttentionType::Critical,
			UserAttentionType::Informational => MillenniumUserAttentionType::Informational
		};
		Self(o)
	}
}

#[derive(Debug, Clone, Default)]
pub struct WindowBuilderWrapper {
	inner: MillenniumWindowBuilder,
	center: bool,
	menu: Option<Menu>
}

// SAFETY: this type is `Send` since `menu_items` are read only here
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}
impl WindowBuilder for WindowBuilderWrapper {
	fn new() -> Self {
		Default::default()
	}

	fn with_config(config: WindowConfig) -> Self {
		let mut window = WindowBuilderWrapper::new()
			.title(config.title.to_string())
			.set_inner_size(config.width, config.height)
			.visible(config.visible)
			.resizable(config.resizable)
			.fullscreen(config.fullscreen)
			.decorations(config.decorations)
			.maximized(config.maximized)
			.always_on_top(config.always_on_top)
			.skip_taskbar(config.skip_taskbar);

		#[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
		{
			window = window.transparent(config.transparent);
		}
		#[cfg(all(target_os = "macos", not(feature = "macos-private-api"), debug_assertions))]
		if config.transparent {
			eprintln!(
				"The window is set to be transparent but the `macos-private-api` is not enabled. This can be enabled via the `millennium.macOSPrivateApi` configuration property."
			);
		}
		#[cfg(target_os = "windows")]
		{
			window = window.titlebar_hidden(config.titlebar_hidden);
		}

		if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
			window = window.min_inner_size(min_width, min_height);
		}
		if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
			window = window.max_inner_size(max_width, max_height);
		}
		if let (Some(x), Some(y)) = (config.x, config.y) {
			window = window.position(x, y);
		}

		if config.center {
			window = window.center();
		}

		window
	}

	fn menu(mut self, menu: Menu) -> Self {
		self.menu.replace(menu);
		self
	}

	fn center(mut self) -> Self {
		self.center = true;
		self
	}

	fn position(mut self, x: f64, y: f64) -> Self {
		self.inner = self.inner.with_position(MillenniumLogicalPosition::new(x, y));
		self
	}

	fn set_inner_size(mut self, width: f64, height: f64) -> Self {
		self.inner = self.inner.with_inner_size(MillenniumLogicalSize::new(width, height));
		self
	}

	fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
		self.inner = self.inner.with_min_inner_size(MillenniumLogicalSize::new(min_width, min_height));
		self
	}

	fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
		self.inner = self.inner.with_max_inner_size(MillenniumLogicalSize::new(max_width, max_height));
		self
	}

	fn resizable(mut self, resizable: bool) -> Self {
		self.inner = self.inner.with_resizable(resizable);
		self
	}

	fn title<S: Into<String>>(mut self, title: S) -> Self {
		self.inner = self.inner.with_title(title.into());
		self
	}

	fn fullscreen(mut self, fullscreen: bool) -> Self {
		self.inner = if fullscreen {
			self.inner.with_fullscreen(Some(Fullscreen::Borderless(None)))
		} else {
			self.inner.with_fullscreen(None)
		};
		self
	}

	/// Deprecated since 0.1.4 (noop)
	/// Windows is automatically focused when created.
	fn focus(self) -> Self {
		self
	}

	fn maximized(mut self, maximized: bool) -> Self {
		self.inner = self.inner.with_maximized(maximized);
		self
	}

	fn visible(mut self, visible: bool) -> Self {
		self.inner = self.inner.with_visible(visible);
		self
	}

	#[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
	fn transparent(mut self, transparent: bool) -> Self {
		self.inner = self.inner.with_transparent(transparent);
		self
	}

	fn decorations(mut self, decorations: bool) -> Self {
		self.inner = self.inner.with_decorations(decorations);
		self
	}

	#[cfg(target_os = "windows")]
	fn titlebar_hidden(mut self, titlebar_hidden: bool) -> Self {
		self.inner = self.inner.with_titlebar_hidden(titlebar_hidden);
		self
	}

	fn always_on_top(mut self, always_on_top: bool) -> Self {
		self.inner = self.inner.with_always_on_top(always_on_top);
		self
	}

	#[cfg(windows)]
	fn parent_window(mut self, parent: HWND) -> Self {
		self.inner = self.inner.with_parent_window(parent);
		self
	}

	#[cfg(target_os = "macos")]
	fn parent_window(mut self, parent: *mut std::ffi::c_void) -> Self {
		use millennium_webview::application::platform::macos::WindowBuilderExtMacOS;
		self.inner = self.inner.with_parent_window(parent);
		self
	}

	#[cfg(windows)]
	fn owner_window(mut self, owner: HWND) -> Self {
		self.inner = self.inner.with_owner_window(owner);
		self
	}

	fn icon(mut self, icon: WindowIcon) -> Result<Self> {
		self.inner = self.inner.with_window_icon(Some(MillenniumIcon::try_from(icon)?.0));
		Ok(self)
	}

	#[cfg(any(windows, target_os = "linux"))]
	fn skip_taskbar(mut self, skip: bool) -> Self {
		self.inner = self.inner.with_skip_taskbar(skip);
		self
	}

	#[cfg(target_os = "macos")]
	fn skip_taskbar(self, _skip: bool) -> Self {
		self
	}

	fn has_icon(&self) -> bool {
		self.inner.window.window_icon.is_some()
	}

	fn get_menu(&self) -> Option<&Menu> {
		self.menu.as_ref()
	}
}

pub struct FileDropEventWrapper(MillenniumFileDropEvent);

impl From<FileDropEventWrapper> for FileDropEvent {
	fn from(event: FileDropEventWrapper) -> Self {
		match event.0 {
			MillenniumFileDropEvent::Hovered(paths) => FileDropEvent::Hovered(paths),
			MillenniumFileDropEvent::Dropped(paths) => FileDropEvent::Dropped(paths),
			// default to cancelled
			// FIXME(maybe): Add `FileDropEvent::Unknown` event?
			_ => FileDropEvent::Cancelled
		}
	}
}

#[cfg(target_os = "macos")]
pub struct NSWindow(*mut std::ffi::c_void);
#[cfg(target_os = "macos")]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for NSWindow {}

#[cfg(windows)]
pub struct Hwnd(HWND);
#[cfg(windows)]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Hwnd {}

#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
pub struct GtkWindow(gtk::ApplicationWindow);
#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GtkWindow {}

#[derive(Debug, Clone)]
pub enum WindowMessage {
	#[cfg(any(debug_assertions, feature = "devtools"))]
	OpenDevTools,
	// Getters
	ScaleFactor(Sender<f64>),
	InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
	OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
	InnerSize(Sender<PhysicalSize<u32>>),
	OuterSize(Sender<PhysicalSize<u32>>),
	IsFullscreen(Sender<bool>),
	IsMaximized(Sender<bool>),
	IsDecorated(Sender<bool>),
	IsResizable(Sender<bool>),
	IsVisible(Sender<bool>),
	IsMenuVisible(Sender<bool>),
	CurrentMonitor(Sender<Option<MonitorHandle>>),
	PrimaryMonitor(Sender<Option<MonitorHandle>>),
	AvailableMonitors(Sender<Vec<MonitorHandle>>),
	#[cfg(target_os = "macos")]
	NSWindow(Sender<NSWindow>),
	#[cfg(windows)]
	Hwnd(Sender<Hwnd>),
	#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
	GtkWindow(Sender<GtkWindow>),
	// Setters
	Center(Sender<Result<()>>),
	RequestUserAttention(Option<UserAttentionTypeWrapper>),
	SetResizable(bool),
	SetTitle(String),
	Maximize,
	Unmaximize,
	Minimize,
	Unminimize,
	ShowMenu,
	HideMenu,
	Show,
	Hide,
	Close,
	SetDecorations(bool),
	SetAlwaysOnTop(bool),
	SetSize(Size),
	SetMinSize(Option<Size>),
	SetMaxSize(Option<Size>),
	SetPosition(Position),
	SetFullscreen(bool),
	SetFocus,
	SetIcon(MillenniumWindowIcon),
	SetSkipTaskbar(bool),
	DragWindow,
	UpdateMenuItem(u16, MenuUpdate),
	RequestRedraw
}

#[derive(Debug, Clone)]
pub enum WebviewMessage {
	EvaluateScript(String),
	#[allow(dead_code)]
	WebviewEvent(WebviewEvent),
	Print
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WebviewEvent {
	Focused(bool)
}

#[cfg(feature = "system-tray")]
#[derive(Debug, Clone)]
pub enum TrayMessage {
	UpdateItem(u16, MenuUpdate),
	UpdateMenu(SystemTrayMenu),
	UpdateIcon(TrayIcon),
	#[cfg(target_os = "macos")]
	UpdateIconAsTemplate(bool),
	Close
}

#[derive(Debug, Clone)]
pub enum GlobalShortcutMessage {
	IsRegistered(Accelerator, Sender<bool>),
	Register(Accelerator, Sender<Result<GlobalShortcutWrapper>>),
	Unregister(GlobalShortcutWrapper, Sender<Result<()>>),
	UnregisterAll(Sender<Result<()>>)
}

#[derive(Debug, Clone)]
pub enum ClipboardMessage {
	WriteText(String, Sender<()>),
	ReadText(Sender<Option<String>>)
}

pub type CreateWebviewClosure<T> = Box<dyn FnOnce(&EventLoopWindowTarget<Message<T>>, &WebContextStore) -> Result<WindowWrapper> + Send>;
pub enum Message<T: 'static> {
	Task(Box<dyn FnOnce() + Send>),
	Window(WebviewId, WindowMessage),
	Webview(WebviewId, WebviewMessage),
	#[cfg(feature = "system-tray")]
	Tray(TrayMessage),
	CreateWebview(WebviewId, CreateWebviewClosure<T>),
	CreateWindow(WebviewId, Box<dyn FnOnce() -> (String, MillenniumWindowBuilder) + Send>, Sender<Result<Weak<Window>>>),
	GlobalShortcut(GlobalShortcutMessage),
	Clipboard(ClipboardMessage),
	UserEvent(T)
}

impl<T: UserEvent> Clone for Message<T> {
	fn clone(&self) -> Self {
		match self {
			Self::Window(i, m) => Self::Window(*i, m.clone()),
			Self::Webview(i, m) => Self::Webview(*i, m.clone()),
			#[cfg(feature = "system-tray")]
			Self::Tray(m) => Self::Tray(m.clone()),
			Self::GlobalShortcut(m) => Self::GlobalShortcut(m.clone()),
			Self::Clipboard(m) => Self::Clipboard(m.clone()),
			Self::UserEvent(t) => Self::UserEvent(t.clone()),
			_ => unimplemented!()
		}
	}
}

#[derive(Debug, Clone)]
pub struct MillenniumDispatcher<T: UserEvent> {
	window_id: WebviewId,
	context: Context<T>
}

// SAFETY: this is safe since the `Context` usage is guarded on
// `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for MillenniumDispatcher<T> {}

impl<T: UserEvent> Dispatch<T> for MillenniumDispatcher<T> {
	type Runtime = MillenniumWebview<T>;
	type WindowBuilder = WindowBuilderWrapper;

	fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
		send_user_message(&self.context, Message::Task(Box::new(f)))
	}

	fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid {
		let id = Uuid::new_v4();
		self.context
			.window_event_listeners
			.lock()
			.unwrap()
			.get(&self.window_id)
			.unwrap()
			.lock()
			.unwrap()
			.insert(id, Box::new(f));
		id
	}

	fn on_menu_event<F: Fn(&MenuEvent) + Send + 'static>(&self, f: F) -> Uuid {
		let id = Uuid::new_v4();
		self.context
			.menu_event_listeners
			.lock()
			.unwrap()
			.get(&self.window_id)
			.unwrap()
			.lock()
			.unwrap()
			.insert(id, Box::new(f));
		id
	}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	fn open_devtools(&self) {
		let _ = send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::OpenDevTools));
	}

	// Getters

	fn scale_factor(&self) -> Result<f64> {
		window_getter!(self, WindowMessage::ScaleFactor)
	}

	fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
		window_getter!(self, WindowMessage::InnerPosition)?
	}

	fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
		window_getter!(self, WindowMessage::OuterPosition)?
	}

	fn inner_size(&self) -> Result<PhysicalSize<u32>> {
		window_getter!(self, WindowMessage::InnerSize)
	}

	fn outer_size(&self) -> Result<PhysicalSize<u32>> {
		window_getter!(self, WindowMessage::OuterSize)
	}

	fn is_fullscreen(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsFullscreen)
	}

	fn is_maximized(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsMaximized)
	}

	/// Gets the window’s current decoration state.
	fn is_decorated(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsDecorated)
	}

	/// Gets the window’s current resizable state.
	fn is_resizable(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsResizable)
	}

	fn is_visible(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsVisible)
	}

	fn is_menu_visible(&self) -> Result<bool> {
		window_getter!(self, WindowMessage::IsMenuVisible)
	}

	fn current_monitor(&self) -> Result<Option<Monitor>> {
		Ok(window_getter!(self, WindowMessage::CurrentMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
	}

	fn primary_monitor(&self) -> Result<Option<Monitor>> {
		Ok(window_getter!(self, WindowMessage::PrimaryMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
	}

	fn available_monitors(&self) -> Result<Vec<Monitor>> {
		Ok(window_getter!(self, WindowMessage::AvailableMonitors)?
			.into_iter()
			.map(|m| MonitorHandleWrapper(m).into())
			.collect())
	}

	#[cfg(target_os = "macos")]
	fn ns_window(&self) -> Result<*mut std::ffi::c_void> {
		window_getter!(self, WindowMessage::NSWindow).map(|w| w.0)
	}

	#[cfg(windows)]
	fn hwnd(&self) -> Result<HWND> {
		window_getter!(self, WindowMessage::Hwnd).map(|w| w.0)
	}

	/// Returns the `ApplicatonWindow` from gtk crate that is used by this
	/// window.
	#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
	fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
		window_getter!(self, WindowMessage::GtkWindow).map(|w| w.0)
	}

	// Setters

	fn center(&self) -> Result<()> {
		window_getter!(self, WindowMessage::Center)?
	}

	fn print(&self) -> Result<()> {
		send_user_message(&self.context, Message::Webview(self.window_id, WebviewMessage::Print))
	}

	fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::RequestUserAttention(request_type.map(Into::into))))
	}

	// Creates a window by dispatching a message to the event loop.
	// Note that this must be called from a separate thread, otherwise the channel
	// will introduce a deadlock.
	fn create_window(&mut self, pending: PendingWindow<T, Self::Runtime>) -> Result<DetachedWindow<T, Self::Runtime>> {
		self.context.create_webview(pending)
	}

	fn set_resizable(&self, resizable: bool) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetResizable(resizable)))
	}

	fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetTitle(title.into())))
	}

	fn maximize(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Maximize))
	}

	fn unmaximize(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Unmaximize))
	}

	fn minimize(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Minimize))
	}

	fn unminimize(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Unminimize))
	}

	fn show_menu(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::ShowMenu))
	}

	fn hide_menu(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::HideMenu))
	}

	fn show(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Show))
	}

	fn hide(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::Hide))
	}

	fn close(&self) -> Result<()> {
		// NOTE: close cannot use the `send_user_message` function because it accesses
		// the event loop callback
		self.context
			.proxy
			.send_event(Message::Window(self.window_id, WindowMessage::Close))
			.map_err(|_| Error::FailedToSendMessage)
	}

	fn set_decorations(&self, decorations: bool) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetDecorations(decorations)))
	}

	fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetAlwaysOnTop(always_on_top)))
	}

	fn set_size(&self, size: Size) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetSize(size)))
	}

	fn set_min_size(&self, size: Option<Size>) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetMinSize(size)))
	}

	fn set_max_size(&self, size: Option<Size>) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetMaxSize(size)))
	}

	fn set_position(&self, position: Position) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetPosition(position)))
	}

	fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetFullscreen(fullscreen)))
	}

	fn set_focus(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetFocus))
	}

	fn set_icon(&self, icon: WindowIcon) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetIcon(MillenniumIcon::try_from(icon)?.0)))
	}

	fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::SetSkipTaskbar(skip)))
	}

	fn start_dragging(&self) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::DragWindow))
	}

	fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
		send_user_message(&self.context, Message::Webview(self.window_id, WebviewMessage::EvaluateScript(script.into())))
	}

	fn update_menu_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
		send_user_message(&self.context, Message::Window(self.window_id, WindowMessage::UpdateMenuItem(id, update)))
	}
}

#[cfg(feature = "system-tray")]
#[derive(Clone, Default)]
struct TrayContext {
	tray: Arc<Mutex<Option<Arc<Mutex<MillenniumSystemTray>>>>>,
	listeners: SystemTrayEventListeners,
	items: SystemTrayItems
}

#[cfg(feature = "system-tray")]
impl fmt::Debug for TrayContext {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TrayContext").field("items", &self.items).finish()
	}
}

enum WindowHandle {
	Webview(WebView),
	Window(Arc<Window>)
}

impl fmt::Debug for WindowHandle {
	fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Ok(())
	}
}

impl WindowHandle {
	fn window(&self) -> &Window {
		match self {
			Self::Webview(w) => w.window(),
			Self::Window(w) => w
		}
	}

	fn inner_size(&self) -> MillenniumPhysicalSize<u32> {
		match self {
			WindowHandle::Window(w) => w.inner_size(),
			WindowHandle::Webview(w) => w.inner_size()
		}
	}
}

#[derive(Debug)]
pub struct WindowWrapper {
	label: String,
	inner: WindowHandle,
	menu_items: Option<HashMap<u16, MillenniumCustomMenuItem>>
}

#[derive(Debug, Clone)]
pub struct EventProxy<T: UserEvent>(MillenniumEventLoopProxy<Message<T>>);

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
	fn send_event(&self, event: T) -> Result<()> {
		self.0.send_event(Message::UserEvent(event)).map_err(|_| Error::EventLoopClosed)
	}
}

pub struct MillenniumWebview<T: UserEvent> {
	main_thread_id: ThreadId,
	global_shortcut_manager: Arc<Mutex<MillenniumShortcutManager>>,
	global_shortcut_manager_handle: GlobalShortcutManagerHandle<T>,
	clipboard_manager: Arc<Mutex<Clipboard>>,
	clipboard_manager_handle: ClipboardManagerWrapper<T>,
	event_loop: EventLoop<Message<T>>,
	windows: Arc<Mutex<HashMap<WebviewId, WindowWrapper>>>,
	webview_id_map: WebviewIdStore,
	web_context: WebContextStore,
	window_event_listeners: WindowEventListeners,
	menu_event_listeners: MenuEventListeners,
	#[cfg(feature = "system-tray")]
	tray_context: TrayContext
}

impl<T: UserEvent> fmt::Debug for MillenniumWebview<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut d = f.debug_struct("MillenniumWebview");
		d.field("main_thread_id", &self.main_thread_id)
			.field("global_shortcut_manager", &self.global_shortcut_manager)
			.field("global_shortcut_manager_handle", &self.global_shortcut_manager_handle)
			.field("clipboard_manager", &self.clipboard_manager)
			.field("clipboard_manager_handle", &self.clipboard_manager_handle)
			.field("event_loop", &self.event_loop)
			.field("windows", &self.windows)
			.field("web_context", &self.web_context);
		#[cfg(feature = "system-tray")]
		d.field("tray_context", &self.tray_context);
		d.finish()
	}
}

/// A handle to the Millennium Webview runtime.
#[derive(Debug, Clone)]
pub struct MillenniumHandle<T: UserEvent> {
	context: Context<T>
}

// SAFETY: this is safe since the `Context` usage is guarded on
// `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for MillenniumHandle<T> {}

impl<T: UserEvent> MillenniumHandle<T> {
	/// Creates a new Millennium Core window using a callback, and returns its
	/// window id.
	pub fn create_core_window<F: FnOnce() -> (String, MillenniumWindowBuilder) + Send + 'static>(&self, f: F) -> Result<Weak<Window>> {
		let (tx, rx) = channel();
		send_user_message(&self.context, Message::CreateWindow(rand::random(), Box::new(f), tx))?;
		rx.recv().unwrap()
	}

	/// Gets the [`WebviewId`] associated with the given [`WindowId`].
	pub fn window_id(&self, window_id: WindowId) -> WebviewId {
		*self.context.webview_id_map.0.lock().unwrap().get(&window_id).unwrap()
	}

	/// Send a message to the event loop.
	pub fn send_event(&self, message: Message<T>) -> Result<()> {
		self.context.proxy.send_event(message).map_err(|_| Error::FailedToSendMessage)?;
		Ok(())
	}
}

impl<T: UserEvent> RuntimeHandle<T> for MillenniumHandle<T> {
	type Runtime = MillenniumWebview<T>;

	fn create_proxy(&self) -> EventProxy<T> {
		EventProxy(self.context.proxy.clone())
	}

	// Creates a window by dispatching a message to the event loop.
	// Note that this must be called from a separate thread, otherwise the channel
	// will introduce a deadlock.
	fn create_window(&self, pending: PendingWindow<T, Self::Runtime>) -> Result<DetachedWindow<T, Self::Runtime>> {
		self.context.create_webview(pending)
	}

	fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
		send_user_message(&self.context, Message::Task(Box::new(f)))
	}

	#[cfg(all(windows, feature = "system-tray"))]
	/// Deprecated. (not needed anymore)
	fn remove_system_tray(&self) -> Result<()> {
		send_user_message(&self.context, Message::Tray(TrayMessage::Close))
	}
}

impl<T: UserEvent> MillenniumWebview<T> {
	fn init(event_loop: EventLoop<Message<T>>) -> Result<Self> {
		let proxy = event_loop.create_proxy();
		let main_thread_id = current_thread().id();
		let web_context = WebContextStore::default();
		let global_shortcut_manager = Arc::new(Mutex::new(MillenniumShortcutManager::new(&event_loop)));
		let clipboard_manager = Arc::new(Mutex::new(Clipboard::new()));
		let windows = Arc::new(Mutex::new(HashMap::default()));
		let webview_id_map = WebviewIdStore::default();
		let window_event_listeners = WindowEventListeners::default();
		let menu_event_listeners = MenuEventListeners::default();

		#[cfg(feature = "system-tray")]
		let tray_context = TrayContext::default();

		let event_loop_context = Context {
			webview_id_map: webview_id_map.clone(),
			main_thread_id,
			proxy,
			window_event_listeners: window_event_listeners.clone(),
			menu_event_listeners: menu_event_listeners.clone(),
			main_thread: DispatcherMainThreadContext {
				window_target: event_loop.deref().clone(),
				web_context: web_context.clone(),
				global_shortcut_manager: global_shortcut_manager.clone(),
				clipboard_manager: clipboard_manager.clone(),
				windows: windows.clone(),
				#[cfg(feature = "system-tray")]
				tray_context: tray_context.clone()
			}
		};

		let global_shortcut_listeners = GlobalShortcutListeners::default();
		let clipboard_manager_handle = ClipboardManagerWrapper { context: event_loop_context.clone() };

		Ok(Self {
			main_thread_id,
			global_shortcut_manager,
			global_shortcut_manager_handle: GlobalShortcutManagerHandle {
				context: event_loop_context,
				shortcuts: Default::default(),
				listeners: global_shortcut_listeners
			},
			clipboard_manager,
			clipboard_manager_handle,
			event_loop,
			windows,
			webview_id_map,
			web_context,
			window_event_listeners,
			menu_event_listeners,
			#[cfg(feature = "system-tray")]
			tray_context
		})
	}
}

impl<T: UserEvent> Runtime<T> for MillenniumWebview<T> {
	type Dispatcher = MillenniumDispatcher<T>;
	type Handle = MillenniumHandle<T>;
	type GlobalShortcutManager = GlobalShortcutManagerHandle<T>;
	type ClipboardManager = ClipboardManagerWrapper<T>;
	#[cfg(feature = "system-tray")]
	type TrayHandler = SystemTrayHandle<T>;
	type EventLoopProxy = EventProxy<T>;

	fn new() -> Result<Self> {
		let event_loop = EventLoop::<Message<T>>::with_user_event();
		Self::init(event_loop)
	}

	#[cfg(any(windows, target_os = "linux"))]
	fn new_any_thread() -> Result<Self> {
		#[cfg(target_os = "linux")]
		use millennium_webview::application::platform::unix::EventLoopExtUnix;
		#[cfg(windows)]
		use millennium_webview::application::platform::windows::EventLoopExtWindows;
		let event_loop = EventLoop::<Message<T>>::new_any_thread();
		Self::init(event_loop)
	}

	fn create_proxy(&self) -> EventProxy<T> {
		EventProxy(self.event_loop.create_proxy())
	}

	fn handle(&self) -> Self::Handle {
		MillenniumHandle {
			context: Context {
				webview_id_map: self.webview_id_map.clone(),
				main_thread_id: self.main_thread_id,
				proxy: self.event_loop.create_proxy(),
				window_event_listeners: self.window_event_listeners.clone(),
				menu_event_listeners: self.menu_event_listeners.clone(),
				main_thread: DispatcherMainThreadContext {
					window_target: self.event_loop.deref().clone(),
					web_context: self.web_context.clone(),
					global_shortcut_manager: self.global_shortcut_manager.clone(),
					clipboard_manager: self.clipboard_manager.clone(),
					windows: self.windows.clone(),
					#[cfg(feature = "system-tray")]
					tray_context: self.tray_context.clone()
				}
			}
		}
	}

	fn global_shortcut_manager(&self) -> Self::GlobalShortcutManager {
		self.global_shortcut_manager_handle.clone()
	}

	fn clipboard_manager(&self) -> Self::ClipboardManager {
		self.clipboard_manager_handle.clone()
	}

	fn create_window(&self, pending: PendingWindow<T, Self>) -> Result<DetachedWindow<T, Self>> {
		let label = pending.label.clone();
		let menu_ids = pending.menu_ids.clone();
		let js_event_listeners = pending.js_event_listeners.clone();
		let proxy = self.event_loop.create_proxy();
		let window_id = rand::random();

		let context = Context {
			webview_id_map: self.webview_id_map.clone(),
			main_thread_id: self.main_thread_id,
			proxy,
			window_event_listeners: self.window_event_listeners.clone(),
			menu_event_listeners: self.menu_event_listeners.clone(),
			main_thread: DispatcherMainThreadContext {
				window_target: self.event_loop.deref().clone(),
				web_context: self.web_context.clone(),
				global_shortcut_manager: self.global_shortcut_manager.clone(),
				clipboard_manager: self.clipboard_manager.clone(),
				windows: self.windows.clone(),
				#[cfg(feature = "system-tray")]
				tray_context: self.tray_context.clone()
			}
		};
		context.prepare_window(window_id);

		let webview = create_webview(window_id, &self.event_loop, &self.web_context, context.clone(), pending)?;

		let dispatcher = MillenniumDispatcher { window_id, context };
		self.windows.lock().unwrap().insert(window_id, webview);

		Ok(DetachedWindow {
			label,
			dispatcher,
			menu_ids,
			js_event_listeners
		})
	}

	#[cfg(feature = "system-tray")]
	fn system_tray(&self, system_tray: SystemTray) -> Result<Self::TrayHandler> {
		let icon = system_tray.icon.expect("tray icon not set").into_platform_icon();

		let mut items = HashMap::new();

		#[cfg(target_os = "macos")]
		let tray = SystemTrayBuilder::new(icon, system_tray.menu.map(|menu| to_millennium_context_menu(&mut items, menu)))
			.with_icon_as_template(system_tray.icon_as_template)
			.build(&self.event_loop)
			.map_err(|e| Error::SystemTray(Box::new(e)))?;

		#[cfg(not(target_os = "macos"))]
		let tray = SystemTrayBuilder::new(icon, system_tray.menu.map(|menu| to_millennium_context_menu(&mut items, menu)))
			.build(&self.event_loop)
			.map_err(|e| Error::SystemTray(Box::new(e)))?;

		*self.tray_context.items.lock().unwrap() = items;
		*self.tray_context.tray.lock().unwrap() = Some(Arc::new(Mutex::new(tray)));

		Ok(SystemTrayHandle {
			proxy: self.event_loop.create_proxy()
		})
	}

	#[cfg(feature = "system-tray")]
	fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid {
		let id = Uuid::new_v4();
		self.tray_context.listeners.lock().unwrap().insert(id, Arc::new(Box::new(f)));
		id
	}

	#[cfg(target_os = "macos")]
	fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
		self.event_loop.set_activation_policy(match activation_policy {
			ActivationPolicy::Regular => MillenniumActivationPolicy::Regular,
			ActivationPolicy::Accessory => MillenniumActivationPolicy::Accessory,
			ActivationPolicy::Prohibited => MillenniumActivationPolicy::Prohibited,
			_ => unimplemented!()
		});
	}

	fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) -> RunIteration {
		use millennium_webview::application::platform::run_return::EventLoopExtRunReturn;
		let windows = self.windows.clone();
		let webview_id_map = self.webview_id_map.clone();
		let web_context = &self.web_context;
		let window_event_listeners = self.window_event_listeners.clone();
		let menu_event_listeners = self.menu_event_listeners.clone();
		#[cfg(feature = "system-tray")]
		let tray_context = self.tray_context.clone();
		let global_shortcut_manager = self.global_shortcut_manager.clone();
		let global_shortcut_manager_handle = self.global_shortcut_manager_handle.clone();
		let clipboard_manager = self.clipboard_manager.clone();
		let mut iteration = RunIteration::default();

		self.event_loop.run_return(|event, event_loop, control_flow| {
			*control_flow = ControlFlow::Wait;
			if let Event::MainEventsCleared = &event {
				*control_flow = ControlFlow::Exit;
			}

			iteration = handle_event_loop(
				event,
				event_loop,
				control_flow,
				EventLoopIterationContext {
					callback: &mut callback,
					windows: windows.clone(),
					webview_id_map: webview_id_map.clone(),
					window_event_listeners: &window_event_listeners,
					global_shortcut_manager: global_shortcut_manager.clone(),
					global_shortcut_manager_handle: &global_shortcut_manager_handle,
					clipboard_manager: clipboard_manager.clone(),
					menu_event_listeners: &menu_event_listeners,
					#[cfg(feature = "system-tray")]
					tray_context: &tray_context
				},
				web_context
			);
		});

		iteration
	}

	fn run<F: FnMut(RunEvent<T>) + 'static>(self, mut callback: F) {
		let windows = self.windows.clone();
		let webview_id_map = self.webview_id_map.clone();
		let web_context = self.web_context;
		let window_event_listeners = self.window_event_listeners.clone();
		let menu_event_listeners = self.menu_event_listeners.clone();
		#[cfg(feature = "system-tray")]
		let tray_context = self.tray_context;
		let global_shortcut_manager = self.global_shortcut_manager.clone();
		let global_shortcut_manager_handle = self.global_shortcut_manager_handle.clone();
		let clipboard_manager = self.clipboard_manager.clone();

		self.event_loop.run(move |event, event_loop, control_flow| {
			handle_event_loop(
				event,
				event_loop,
				control_flow,
				EventLoopIterationContext {
					callback: &mut callback,
					webview_id_map: webview_id_map.clone(),
					windows: windows.clone(),
					window_event_listeners: &window_event_listeners,
					global_shortcut_manager: global_shortcut_manager.clone(),
					global_shortcut_manager_handle: &global_shortcut_manager_handle,
					clipboard_manager: clipboard_manager.clone(),
					menu_event_listeners: &menu_event_listeners,
					#[cfg(feature = "system-tray")]
					tray_context: &tray_context
				},
				&web_context
			);
		})
	}
}

pub struct EventLoopIterationContext<'a, T: UserEvent> {
	callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
	webview_id_map: WebviewIdStore,
	windows: Arc<Mutex<HashMap<WebviewId, WindowWrapper>>>,
	window_event_listeners: &'a WindowEventListeners,
	global_shortcut_manager: Arc<Mutex<MillenniumShortcutManager>>,
	global_shortcut_manager_handle: &'a GlobalShortcutManagerHandle<T>,
	clipboard_manager: Arc<Mutex<Clipboard>>,
	menu_event_listeners: &'a MenuEventListeners,
	#[cfg(feature = "system-tray")]
	tray_context: &'a TrayContext
}

struct UserMessageContext<'a> {
	window_event_listeners: &'a WindowEventListeners,
	global_shortcut_manager: Arc<Mutex<MillenniumShortcutManager>>,
	clipboard_manager: Arc<Mutex<Clipboard>>,
	menu_event_listeners: &'a MenuEventListeners,
	windows: Arc<Mutex<HashMap<WebviewId, WindowWrapper>>>,
	#[cfg(feature = "system-tray")]
	tray_context: &'a TrayContext
}

fn handle_user_message<T: UserEvent>(
	event_loop: &EventLoopWindowTarget<Message<T>>,
	message: Message<T>,
	context: UserMessageContext<'_>,
	web_context: &WebContextStore
) -> RunIteration {
	let UserMessageContext {
		window_event_listeners,
		menu_event_listeners,
		global_shortcut_manager,
		clipboard_manager,
		windows,
		#[cfg(feature = "system-tray")]
		tray_context
	} = context;
	match message {
		Message::Task(task) => task(),
		Message::Window(id, window_message) => {
			if let Some(webview) = windows.lock().expect("poisoned webview collection").get_mut(&id) {
				let window = webview.inner.window();
				match window_message {
					#[cfg(any(debug_assertions, feature = "devtools"))]
					WindowMessage::OpenDevTools => {
						if let WindowHandle::Webview(w) = &webview.inner {
							w.devtool();
						}
					}
					// Getters
					WindowMessage::ScaleFactor(tx) => tx.send(window.scale_factor()).unwrap(),
					WindowMessage::InnerPosition(tx) => tx
						.send(
							window
								.inner_position()
								.map(|p| PhysicalPositionWrapper(p).into())
								.map_err(|_| Error::FailedToSendMessage)
						)
						.unwrap(),
					WindowMessage::OuterPosition(tx) => tx
						.send(
							window
								.outer_position()
								.map(|p| PhysicalPositionWrapper(p).into())
								.map_err(|_| Error::FailedToSendMessage)
						)
						.unwrap(),
					WindowMessage::InnerSize(tx) => tx.send(PhysicalSizeWrapper(webview.inner.inner_size()).into()).unwrap(),
					WindowMessage::OuterSize(tx) => tx.send(PhysicalSizeWrapper(window.outer_size()).into()).unwrap(),
					WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
					WindowMessage::IsMaximized(tx) => tx.send(window.is_maximized()).unwrap(),
					WindowMessage::IsDecorated(tx) => tx.send(window.is_decorated()).unwrap(),
					WindowMessage::IsResizable(tx) => tx.send(window.is_resizable()).unwrap(),
					WindowMessage::IsVisible(tx) => tx.send(window.is_visible()).unwrap(),
					WindowMessage::IsMenuVisible(tx) => tx.send(window.is_menu_visible()).unwrap(),
					WindowMessage::CurrentMonitor(tx) => tx.send(window.current_monitor()).unwrap(),
					WindowMessage::PrimaryMonitor(tx) => tx.send(window.primary_monitor()).unwrap(),
					WindowMessage::AvailableMonitors(tx) => tx.send(window.available_monitors().collect()).unwrap(),
					#[cfg(target_os = "macos")]
					WindowMessage::NSWindow(tx) => tx.send(NSWindow(window.ns_window())).unwrap(),
					#[cfg(windows)]
					WindowMessage::Hwnd(tx) => tx.send(Hwnd(HWND(window.hwnd() as _))).unwrap(),
					#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
					WindowMessage::GtkWindow(tx) => tx.send(GtkWindow(window.gtk_window().clone())).unwrap(),
					// Setters
					WindowMessage::Center(tx) => {
						tx.send(center_window(window, webview.inner.inner_size())).unwrap();
					}
					WindowMessage::RequestUserAttention(request_type) => {
						window.request_user_attention(request_type.map(|r| r.0));
					}
					WindowMessage::SetResizable(resizable) => window.set_resizable(resizable),
					WindowMessage::SetTitle(title) => window.set_title(&title),
					WindowMessage::Maximize => window.set_maximized(true),
					WindowMessage::Unmaximize => window.set_maximized(false),
					WindowMessage::Minimize => window.set_minimized(true),
					WindowMessage::Unminimize => window.set_minimized(false),
					WindowMessage::ShowMenu => window.show_menu(),
					WindowMessage::HideMenu => window.hide_menu(),
					WindowMessage::Show => window.set_visible(true),
					WindowMessage::Hide => window.set_visible(false),
					WindowMessage::Close => panic!("cannot handle `WindowMessage::Close` on the main thread"),
					WindowMessage::SetDecorations(decorations) => window.set_decorations(decorations),
					WindowMessage::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
					WindowMessage::SetSize(size) => {
						window.set_inner_size(SizeWrapper::from(size).0);
					}
					WindowMessage::SetMinSize(size) => {
						window.set_min_inner_size(size.map(|s| SizeWrapper::from(s).0));
					}
					WindowMessage::SetMaxSize(size) => {
						window.set_max_inner_size(size.map(|s| SizeWrapper::from(s).0));
					}
					WindowMessage::SetPosition(position) => window.set_outer_position(PositionWrapper::from(position).0),
					WindowMessage::SetFullscreen(fullscreen) => {
						if fullscreen {
							window.set_fullscreen(Some(Fullscreen::Borderless(None)))
						} else {
							window.set_fullscreen(None)
						}
					}
					WindowMessage::SetFocus => {
						window.set_focus();
					}
					WindowMessage::SetIcon(icon) => {
						window.set_window_icon(Some(icon));
					}
					WindowMessage::SetSkipTaskbar(_skip) => {
						#[cfg(any(windows, target_os = "linux"))]
						window.set_skip_taskbar(_skip);
					}
					WindowMessage::DragWindow => {
						let _ = window.drag_window();
					}
					WindowMessage::UpdateMenuItem(id, update) => {
						if let Some(menu_items) = webview.menu_items.as_mut() {
							let item = menu_items.get_mut(&id).expect("menu item not found");
							match update {
								MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
								MenuUpdate::SetTitle(title) => item.set_title(&title),
								MenuUpdate::SetSelected(selected) => item.set_selected(selected),
								#[cfg(target_os = "macos")]
								MenuUpdate::SetNativeImage(image) => item.set_native_image(NativeImageWrapper::from(image).0)
							}
						}
					}
					WindowMessage::RequestRedraw => {
						window.request_redraw();
					}
				}
			}
		}
		Message::Webview(id, webview_message) => match webview_message {
			WebviewMessage::EvaluateScript(script) => {
				if let Some(WindowHandle::Webview(webview)) = windows.lock().expect("poisoned webview collection").get(&id).map(|w| &w.inner) {
					if let Err(e) = webview.evaluate_script(&script) {
						#[cfg(debug_assertions)]
						eprintln!("{}", e);
					}
				}
			}
			WebviewMessage::Print => {
				if let Some(WindowHandle::Webview(webview)) = windows.lock().expect("poisoned webview collection").get(&id).map(|w| &w.inner) {
					let _ = webview.print();
				}
			}
			WebviewMessage::WebviewEvent(event) => {
				if let Some(event) = WindowEventWrapper::from(&event).0 {
					for handler in window_event_listeners.lock().unwrap().get(&id).unwrap().lock().unwrap().values() {
						handler(&event);
					}
				}
			}
		},
		Message::CreateWebview(window_id, handler) => match handler(event_loop, web_context) {
			Ok(webview) => {
				windows.lock().expect("poisoned webview collection").insert(window_id, webview);
			}
			Err(e) => {
				#[cfg(debug_assertions)]
				eprintln!("{}", e);
			}
		},
		Message::CreateWindow(window_id, handler, sender) => {
			let (label, builder) = handler();
			if let Ok(window) = builder.build(event_loop) {
				window_event_listeners
					.lock()
					.unwrap()
					.insert(window_id, WindowEventListenersMap::default());

				menu_event_listeners
					.lock()
					.unwrap()
					.insert(window_id, WindowMenuEventListeners::default());

				let w = Arc::new(window);

				windows.lock().expect("poisoned webview collection").insert(
					window_id,
					WindowWrapper {
						label,
						inner: WindowHandle::Window(w.clone()),
						menu_items: Default::default()
					}
				);
				sender.send(Ok(Arc::downgrade(&w))).unwrap();
			} else {
				sender.send(Err(Error::CreateWindow)).unwrap();
			}
		}

		#[cfg(feature = "system-tray")]
		Message::Tray(tray_message) => match tray_message {
			TrayMessage::UpdateItem(menu_id, update) => {
				let mut tray = tray_context.items.as_ref().lock().unwrap();
				let item = tray.get_mut(&menu_id).expect("menu item not found");
				match update {
					MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
					MenuUpdate::SetTitle(title) => item.set_title(&title),
					MenuUpdate::SetSelected(selected) => item.set_selected(selected),
					#[cfg(target_os = "macos")]
					MenuUpdate::SetNativeImage(image) => item.set_native_image(NativeImageWrapper::from(image).0)
				}
			}
			TrayMessage::UpdateMenu(menu) => {
				if let Some(tray) = &*tray_context.tray.lock().unwrap() {
					let mut items = HashMap::new();
					tray.lock().unwrap().set_menu(&to_millennium_context_menu(&mut items, menu));
					*tray_context.items.lock().unwrap() = items;
				}
			}
			TrayMessage::UpdateIcon(icon) => {
				if let Some(tray) = &*tray_context.tray.lock().unwrap() {
					tray.lock().unwrap().set_icon(icon.into_platform_icon());
				}
			}
			#[cfg(target_os = "macos")]
			TrayMessage::UpdateIconAsTemplate(is_template) => {
				if let Some(tray) = &*tray_context.tray.lock().unwrap() {
					tray.lock().unwrap().set_icon_as_template(is_template);
				}
			}
			TrayMessage::Close => {
				*tray_context.tray.lock().unwrap() = None;
				tray_context.listeners.lock().unwrap().clear();
				tray_context.items.lock().unwrap().clear();
			}
		},
		Message::GlobalShortcut(message) => match message {
			GlobalShortcutMessage::IsRegistered(accelerator, tx) => tx.send(global_shortcut_manager.lock().unwrap().is_registered(&accelerator)).unwrap(),
			GlobalShortcutMessage::Register(accelerator, tx) => tx
				.send(
					global_shortcut_manager
						.lock()
						.unwrap()
						.register(accelerator)
						.map(GlobalShortcutWrapper)
						.map_err(|e| Error::GlobalShortcut(Box::new(e)))
				)
				.unwrap(),
			GlobalShortcutMessage::Unregister(shortcut, tx) => tx
				.send(
					global_shortcut_manager
						.lock()
						.unwrap()
						.unregister(shortcut.0)
						.map_err(|e| Error::GlobalShortcut(Box::new(e)))
				)
				.unwrap(),
			GlobalShortcutMessage::UnregisterAll(tx) => tx
				.send(
					global_shortcut_manager
						.lock()
						.unwrap()
						.unregister_all()
						.map_err(|e| Error::GlobalShortcut(Box::new(e)))
				)
				.unwrap()
		},
		Message::Clipboard(message) => match message {
			ClipboardMessage::WriteText(text, tx) => {
				clipboard_manager.lock().unwrap().write_text(text);
				tx.send(()).unwrap();
			}
			ClipboardMessage::ReadText(tx) => tx.send(clipboard_manager.lock().unwrap().read_text()).unwrap()
		},
		Message::UserEvent(_) => ()
	}

	let it = RunIteration {
		window_count: windows.lock().expect("poisoned webview collection").len()
	};
	it
}

fn handle_event_loop<T: UserEvent>(
	event: Event<'_, Message<T>>,
	event_loop: &EventLoopWindowTarget<Message<T>>,
	control_flow: &mut ControlFlow,
	context: EventLoopIterationContext<'_, T>,
	web_context: &WebContextStore
) -> RunIteration {
	let EventLoopIterationContext {
		callback,
		webview_id_map,
		windows,
		window_event_listeners,
		global_shortcut_manager,
		global_shortcut_manager_handle,
		clipboard_manager,
		menu_event_listeners,
		#[cfg(feature = "system-tray")]
		tray_context
	} = context;
	if *control_flow != ControlFlow::Exit {
		*control_flow = ControlFlow::Wait;
	}

	match event {
		Event::NewEvents(StartCause::Init) => {
			callback(RunEvent::Ready);
		}

		Event::NewEvents(StartCause::Poll) => {
			callback(RunEvent::Resumed);
		}

		Event::MainEventsCleared => {
			callback(RunEvent::MainEventsCleared);
		}

		Event::LoopDestroyed => {
			callback(RunEvent::Exit);
		}

		Event::GlobalShortcutEvent(accelerator_id) => {
			for (id, handler) in &*global_shortcut_manager_handle.listeners.lock().unwrap() {
				if accelerator_id == *id {
					handler();
				}
			}
		}
		Event::MenuEvent {
			window_id,
			menu_id,
			origin: MenuType::MenuBar,
			..
		} => {
			let window_id = window_id.unwrap(); // always Some on MenuBar event
			let event = MenuEvent { menu_item_id: menu_id.0 };
			let window_menu_event_listeners = {
				let window_id = webview_id_map.get(&window_id);
				let listeners = menu_event_listeners.lock().unwrap();
				listeners.get(&window_id).cloned().unwrap_or_default()
			};
			for handler in window_menu_event_listeners.lock().unwrap().values() {
				handler(&event);
			}
		}
		#[cfg(feature = "system-tray")]
		Event::MenuEvent {
			window_id: _,
			menu_id,
			origin: MenuType::ContextMenu,
			..
		} => {
			let event = SystemTrayEvent::MenuItemClick(menu_id.0);
			let listeners = tray_context.listeners.lock().unwrap().clone();
			for handler in listeners.values() {
				handler(&event);
			}
		}
		#[cfg(feature = "system-tray")]
		Event::TrayEvent {
			bounds,
			event,
			position: _cursor_position,
			..
		} => {
			let (position, size) = (PhysicalPositionWrapper(bounds.position).into(), PhysicalSizeWrapper(bounds.size).into());
			let event = match event {
				TrayEvent::RightClick => SystemTrayEvent::RightClick { position, size },
				TrayEvent::DoubleClick => SystemTrayEvent::DoubleClick { position, size },
				// default to left click
				_ => SystemTrayEvent::LeftClick { position, size }
			};
			for handler in tray_context.listeners.lock().unwrap().values() {
				handler(&event);
			}
		}
		Event::WindowEvent { event, window_id, .. } => {
			let window_id = webview_id_map.get(&window_id);
			// NOTE(amrbashir): we handle this event here instead of `match` statement below
			// because we want to focus the webview as soon as possible, especially on
			// windows.
			if event == MillenniumWindowEvent::Focused(true) {
				if let Some(WindowHandle::Webview(webview)) = windows.lock().expect("poisoned webview collection").get(&window_id).map(|w| &w.inner) {
					if webview.window().is_visible() {
						webview.focus();
					}
				}
			}

			{
				let windows_lock = windows.lock().expect("poisoned webview collection");
				if let Some(window_handle) = windows_lock.get(&window_id).map(|w| &w.inner) {
					if let Some(event) = WindowEventWrapper::parse(window_handle, &event).0 {
						drop(windows_lock);
						for handler in window_event_listeners.lock().unwrap().get(&window_id).unwrap().lock().unwrap().values() {
							handler(&event);
						}
					}
				}
			}

			match event {
				MillenniumWindowEvent::CloseRequested => {
					on_close_requested(callback, window_id, windows.clone(), control_flow, window_event_listeners, menu_event_listeners.clone());
				}
				MillenniumWindowEvent::Resized(_) => {
					if let Some(WindowHandle::Webview(webview)) = windows.lock().expect("poisoned webview collection").get(&window_id).map(|w| &w.inner) {
						if let Err(e) = webview.resize() {
							#[cfg(debug_assertions)]
							eprintln!("{}", e);
						}
					}
				}
				_ => {}
			}
		}
		Event::UserEvent(message) => match message {
			Message::Window(id, WindowMessage::Close) => {
				on_window_close(
					callback,
					id,
					windows.lock().expect("poisoned webview collection"),
					control_flow,
					#[cfg(target_os = "linux")]
					window_event_listeners,
					menu_event_listeners.clone()
				);
			}
			Message::UserEvent(t) => callback(RunEvent::UserEvent(t)),
			message => {
				return handle_user_message(
					event_loop,
					message,
					UserMessageContext {
						window_event_listeners,
						global_shortcut_manager,
						clipboard_manager,
						menu_event_listeners,
						windows,
						#[cfg(feature = "system-tray")]
						tray_context
					},
					web_context
				);
			}
		},
		_ => ()
	}

	let it = RunIteration {
		window_count: windows.lock().expect("poisoned webview collection").len()
	};
	it
}

fn on_close_requested<'a, T: UserEvent>(
	callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
	window_id: WebviewId,
	windows: Arc<Mutex<HashMap<WebviewId, WindowWrapper>>>,
	control_flow: &mut ControlFlow,
	window_event_listeners: &WindowEventListeners,
	menu_event_listeners: MenuEventListeners
) -> Option<WindowWrapper> {
	let (tx, rx) = channel();
	let windows_guard = windows.lock().expect("poisoned webview collection");
	if let Some(w) = windows_guard.get(&window_id) {
		let label = w.label.clone();
		drop(windows_guard);
		for handler in window_event_listeners.lock().unwrap().get(&window_id).unwrap().lock().unwrap().values() {
			handler(&WindowEvent::CloseRequested {
				label: label.clone(),
				signal_tx: tx.clone()
			});
		}
		callback(RunEvent::CloseRequested { label, signal_tx: tx });
		if let Ok(true) = rx.try_recv() {
			None
		} else {
			on_window_close(
				callback,
				window_id,
				windows.lock().expect("poisoned webview collection"),
				control_flow,
				#[cfg(target_os = "linux")]
				window_event_listeners,
				menu_event_listeners
			)
		}
	} else {
		None
	}
}

fn on_window_close<'a, T: UserEvent>(
	callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
	window_id: WebviewId,
	mut windows: MutexGuard<'a, HashMap<WebviewId, WindowWrapper>>,
	control_flow: &mut ControlFlow,
	#[cfg(target_os = "linux")] window_event_listeners: &WindowEventListeners,
	menu_event_listeners: MenuEventListeners
) -> Option<WindowWrapper> {
	#[allow(unused_mut)]
	let w = if let Some(mut webview) = windows.remove(&window_id) {
		let is_empty = windows.is_empty();
		drop(windows);
		menu_event_listeners.lock().unwrap().remove(&window_id);
		callback(RunEvent::WindowClose(webview.label.clone()));

		if is_empty {
			let (tx, rx) = channel();
			callback(RunEvent::ExitRequested {
				window_label: webview.label.clone(),
				tx
			});

			let recv = rx.try_recv();
			let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

			if !should_prevent {
				*control_flow = ControlFlow::Exit;
			}
		}
		Some(webview)
	} else {
		None
	};
	// TODO: Millennium Core does not fire the destroyed event properly
	#[cfg(target_os = "linux")]
	{
		for handler in window_event_listeners.lock().unwrap().get(&window_id).unwrap().lock().unwrap().values() {
			handler(&WindowEvent::Destroyed);
		}
	}
	w
}

fn center_window(window: &Window, window_size: MillenniumPhysicalSize<u32>) -> Result<()> {
	if let Some(monitor) = window.current_monitor() {
		let screen_size = monitor.size();
		let x = (screen_size.width as i32 - window_size.width as i32) / 2;
		let y = (screen_size.height as i32 - window_size.height as i32) / 2;
		window.set_outer_position(MillenniumPhysicalPosition::new(x, y));
		Ok(())
	} else {
		Err(Error::FailedToGetMonitor)
	}
}

fn to_millennium_menu(custom_menu_items: &mut HashMap<MenuHash, MillenniumCustomMenuItem>, menu: Menu) -> MenuBar {
	let mut millennium_menu = MenuBar::new();
	for item in menu.items {
		match item {
			MenuEntry::CustomItem(c) => {
				let mut attributes = MenuItemAttributesWrapper::from(&c).0;
				attributes = attributes.with_id(MillenniumMenuId(c.id));
				#[allow(unused_mut)]
				let mut item = millennium_menu.add_item(attributes);
				#[cfg(target_os = "macos")]
				if let Some(native_image) = c.native_image {
					item.set_native_image(NativeImageWrapper::from(native_image).0);
				}
				custom_menu_items.insert(c.id, item);
			}
			MenuEntry::NativeItem(i) => {
				millennium_menu.add_native_item(MenuItemWrapper::from(i).0);
			}
			MenuEntry::Submenu(submenu) => {
				millennium_menu.add_submenu(&submenu.title, submenu.enabled, to_millennium_menu(custom_menu_items, submenu.inner));
			}
		}
	}
	millennium_menu
}

fn create_webview<T: UserEvent>(
	window_id: WebviewId,
	event_loop: &EventLoopWindowTarget<Message<T>>,
	web_context: &WebContextStore,
	context: Context<T>,
	pending: PendingWindow<T, MillenniumWebview<T>>
) -> Result<WindowWrapper> {
	#[allow(unused_mut)]
	let PendingWindow {
		webview_attributes,
		uri_scheme_protocols,
		mut window_builder,
		ipc_handler,
		label,
		url,
		menu_ids,
		js_event_listeners,
		..
	} = pending;
	let webview_id_map = context.webview_id_map.clone();
	#[cfg(all(windows, not(feature = "__rust_analyzer_hack")))]
	let proxy = context.proxy.clone();

	let is_window_transparent = window_builder.inner.window.transparent;
	let menu_items = if let Some(menu) = window_builder.menu {
		let mut menu_items = HashMap::new();
		let menu = to_millennium_menu(&mut menu_items, menu);
		window_builder.inner = window_builder.inner.with_menu(menu);
		Some(menu_items)
	} else {
		None
	};
	let window = window_builder.inner.build(event_loop).unwrap();

	if window_builder.center {
		let _ = center_window(&window, window.inner_size());
	}
	let mut webview_builder = WebViewBuilder::new(window)
		.map_err(|e| Error::CreateWebview(Box::new(e)))?
		.with_url(&url)
		.unwrap() // safe to unwrap because we validate the URL beforehand
		.with_transparent(is_window_transparent);
	if webview_attributes.file_drop_handler_enabled {
		webview_builder = webview_builder.with_file_drop_handler(create_file_drop_handler(&context));
	}
	if let Some(handler) = ipc_handler {
		webview_builder = webview_builder.with_ipc_handler(create_ipc_handler(context, label.clone(), menu_ids, js_event_listeners, handler));
	}
	for (scheme, protocol) in uri_scheme_protocols {
		webview_builder = webview_builder.with_custom_protocol(scheme, move |millennium_request| {
			protocol(&HttpRequestWrapper::from(millennium_request).0)
				.map(|millennium_response| HttpResponseWrapper::from(millennium_response).0)
				.map_err(|_| millennium_webview::Error::InitScriptError)
		});
	}

	for script in webview_attributes.initialization_scripts {
		webview_builder = webview_builder.with_initialization_script(&script);
	}

	let mut web_context = web_context.lock().expect("poisoned WebContext store");
	let is_first_context = web_context.is_empty();
	let automation_enabled = std::env::var("MILLENNIUM_AUTOMATION").as_deref() == Ok("true");
	let web_context = match web_context.entry(
		// force a unique WebContext when automation is false;
		// the context must be stored on the HashMap because it must outlive the WebView on macOS
		if automation_enabled {
			webview_attributes.data_directory.clone()
		} else {
			// random unique key
			Some(Uuid::new_v4().to_hyphenated().to_string().into())
		}
	) {
		Occupied(occupied) => occupied.into_mut(),
		Vacant(vacant) => {
			let mut web_context = WebContext::new(webview_attributes.data_directory);
			web_context.set_allows_automation(if automation_enabled { is_first_context } else { false });
			vacant.insert(web_context)
		}
	};

	if webview_attributes.clipboard {
		webview_builder.webview.clipboard = true;
	}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	{
		webview_builder = webview_builder.with_dev_tool(true);
	}

	let webview = webview_builder
		.with_web_context(web_context)
		.build()
		.map_err(|e| Error::CreateWebview(Box::new(e)))?;

	webview_id_map.insert(webview.window().id(), window_id);

	#[cfg(all(windows, not(feature = "__rust_analyzer_hack")))]
	{
		let controller = webview.controller();
		let proxy_ = proxy.clone();
		let mut token = EventRegistrationToken::default();
		unsafe {
			controller.GotFocus(
				FocusChangedEventHandler::create(Box::new(move |_, _| {
					let _ = proxy_.send_event(Message::Webview(window_id, WebviewMessage::WebviewEvent(WebviewEvent::Focused(true))));
					Ok(())
				})),
				&mut token
			)
		}
		.unwrap();
		unsafe {
			controller.LostFocus(
				FocusChangedEventHandler::create(Box::new(move |_, _| {
					let _ = proxy.send_event(Message::Webview(window_id, WebviewMessage::WebviewEvent(WebviewEvent::Focused(false))));
					Ok(())
				})),
				&mut token
			)
		}
		.unwrap();
	}

	Ok(WindowWrapper {
		label,
		inner: WindowHandle::Webview(webview),
		menu_items
	})
}

/// Create a Millennium Webview ipc handler from a Millennium ipc handler.
fn create_ipc_handler<T: UserEvent>(
	context: Context<T>,
	label: String,
	menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
	js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
	handler: WebviewIpcHandler<T, MillenniumWebview<T>>
) -> Box<dyn Fn(&Window, String) + 'static> {
	Box::new(move |window, request| {
		handler(
			DetachedWindow {
				dispatcher: MillenniumDispatcher {
					window_id: *context.webview_id_map.0.lock().unwrap().get(&window.id()).unwrap(),
					context: context.clone()
				},
				label: label.clone(),
				menu_ids: menu_ids.clone(),
				js_event_listeners: js_event_listeners.clone()
			},
			request
		);
	})
}

/// Create a Millennium Webview file drop handler.
fn create_file_drop_handler<T: UserEvent>(context: &Context<T>) -> Box<dyn Fn(&Window, MillenniumFileDropEvent) -> bool + 'static> {
	let window_event_listeners = context.window_event_listeners.clone();
	let webview_id_map = context.webview_id_map.clone();
	Box::new(move |window, event| {
		let event: FileDropEvent = FileDropEventWrapper(event).into();
		let window_event = WindowEvent::FileDrop(event);
		let listeners = window_event_listeners.lock().unwrap();
		if let Some(window_listeners) = listeners.get(&webview_id_map.get(&window.id())) {
			let listeners_map = window_listeners.lock().unwrap();
			let has_listener = !listeners_map.is_empty();
			for listener in listeners_map.values() {
				listener(&window_event);
			}
			// block the default OS action on file drop if we had a listener
			has_listener
		} else {
			false
		}
	})
}
