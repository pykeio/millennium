/**
 * Copyright 2022 pyke.io
 *           2019-2021 Tauri Programme within The Commons Conservancy
 *                     [https://tauri.studio/]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Access the Rust HTTP client.
 *
 * The APIs must be allowlisted in `.millenniumrc`:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"http": {
 * 				"all": true, // enable all HTTP APIs
 * 				"request": true // enable all request APIs
 * 			}
 * 		}
 * 	}
 * }
 * ```
 *
 * ## Security
 * This API has a scope configuration that forces you to restrict the URLs and paths that can be accessed using glob patterns.
 *
 * For instance, this scope configuration only allows making HTTP requests to only paths on google.com:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"http": {
 * 				"scope": [ "https://*.google.com/*" ]
 * 			}
 * 		}
 * 	}
 * }
 * ```
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';

export interface Duration {
	secs: number;
	nanos: number;
}

interface ClientOptions {
	maxRedirections?: number;
	connectTimeout?: number | Duration;
}

export enum ResponseType {
	JSON = 1,
	TEXT = 2,
	BINARY = 3
}

interface FilePart<T> {
	file: string | T;
	mime?: string;
	fileName?: string;
}

type Part = string | Uint8Array | FilePart<Uint8Array>;
type BodyType = 'Form' | 'Json' | 'Text' | 'Bytes';

export class Body {
	private constructor(public readonly type: BodyType, public readonly payload: unknown) {}

	/**
	 * Creates a new form data body. The form data is an object where each key is the entry name,
	 * and the value is either a string or a file object.
	 *
	 * By default, it sets the `application/x-www-form-urlencoded` Content-Type header, but you can
	 * set it to `multipart/form-data` if the Cargo feature `http-multipart` is enabled.
	 *
	 * Note that a file path must be allowed in the `fs` scope configuration.
	 *
	 * @param data The body data.
	 */
	public static form(data: Record<string, Part>): Body {
		const form: Record<string, string | number[] | FilePart<number[]>> = {};
		for (const key in data) {
			const v = data[key];
			form[key] = typeof v === 'string'
				? v
				: v instanceof Uint8Array || Array.isArray(v)
					? Array.from(v)
					: typeof v.file === 'string'
						? { file: v.file, mime: v.mime, fileName: v.fileName }
						: { file: Array.from(v.file), mime: v.mime, fileName: v.fileName };
		}
		return new Body('Form', form);
	}

	/**
	 * Creates a new JSON body.
	 *
	 * @param data The body JSON object.
	 */
	public static json(data: Record<any, any>): Body {
		return new Body('Json', data);
	}

	/**
	 * Creates a new UTF-8 string body.
	 *
	 * @param data The body string.
	 */
	public static text(data: string): Body {
		return new Body('Text', data);
	}

	/**
	 * Creates a new raw byte array body.
	 *
	 * @param data The body byte array.
	 */
	public static binary(data: Uint8Array): Body {
		return new Body('Bytes', Array.from(data));
	}
}

export type HttpVerb =
	| 'GET'
	| 'POST'
	| 'PUT'
	| 'DELETE'
	| 'PATCH'
	| 'HEAD'
	| 'OPTIONS'
	| 'CONNECT'
	| 'TRACE';

export interface HttpOptions {
	method: HttpVerb;
	url: string;
	headers?: Record<string, any>;
	query?: Record<string, any>;
	body?: Body;
	timeout?: number | Duration;
	responseType?: ResponseType;
}

export type RequestOptions = Omit<HttpOptions, 'method' | 'url'>;
export type FetchOptions = Omit<HttpOptions, 'url'>;

interface IResponse<T> {
	url: string;
	status: number;
	headers: Record<string, string>;
	rawHeaders: Record<string, string[]>;
	data: T;
}

export class Response<T> {
	public readonly url: string;
	/** The HTTP status code. */
	public readonly status: number;
	/** A boolean indicating whether the response was successful (status in the range 200-299). */
	public readonly ok: boolean;
	/** The response headers. */
	public readonly headers: Record<string, string>;
	/** The raw response headers. */
	public readonly rawHeaders: Record<string, string[]>;
	/** The response body. */
	public readonly data: T;

	public constructor(response: IResponse<T>) {
		this.url = response.url;
		this.status = response.status;
		this.ok = response.status >= 200 && response.status < 300;
		this.headers = response.headers;
		this.rawHeaders = response.rawHeaders;
		this.data = response.data;
	}
}

class Client {
	public constructor(public readonly id: number) {}

	/** Drops the client instance. */
	public async drop(): Promise<void> {
		return invokeMillenniumCommand({
			__millenniumModule: 'Http',
			message: {
				cmd: 'dropClient',
				client: this.id
			}
		});
	}

	/** Makes an HTTP request. */
	public async request<T>(options: HttpOptions): Promise<Response<T>> {
		const jsonResponse = !options.responseType || options.responseType === ResponseType.JSON;
		if (jsonResponse)
			options.responseType = ResponseType.TEXT;

		const res = await invokeMillenniumCommand<IResponse<T>>({
			__millenniumModule: 'Http',
			message: {
				cmd: 'httpRequest',
				client: this.id,
				options
			}
		});
		const response = new Response(res);
		if (jsonResponse)
			try {
				// @ts-expect-error
				response.data = JSON.parse(response.data as unknown as string);
			} catch (e) {
				if (response.ok && (response.data as unknown as string) === '')
					// @ts-expect-error
					response.data = {};
				else if (response.ok)
					throw new Error(`Failed to parse response body (\`${response.data}\`): ${e}\nTry setting the responseType to a different type if the API doesn't return JSON.`);
			}

		return response;
	}

	/** Makes a GET request. */
	public async get<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
		return this.request({ ...options, method: 'GET', url });
	}

	/** Makes a POST request. */
	public async post<T>(url: string, body?: Body, options?: RequestOptions): Promise<Response<T>> {
		return this.request({ ...options, method: 'POST', body, url });
	}

	/** Makes a PUT request. */
	public async put<T>(url: string, body?: Body, options?: RequestOptions): Promise<Response<T>> {
		return this.request({ ...options, method: 'PUT', body, url });
	}

	/** Makes a PATCH request. */
	public async patch<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
		return this.request({ ...options, method: 'PATCH', url });
	}

	/** Makes a DELETE request. */
	public async delete<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
		return this.request({ ...options, method: 'DELETE', url });
	}
}

/**
 * Creates a new client with the specified options.
 *
 * @param options Client configuration.
 */
export async function createClient(options?: ClientOptions): Promise<Client> {
	const id = await invokeMillenniumCommand<number>({
		__millenniumModule: 'Http',
		message: {
			cmd: 'createClient',
			options
		}
	});
	return new Client(id);
}

/** @internal */
let defaultClient: Client | null = null;

/**
 * Perform an asynchronous HTTP request using the default client.
 */
export async function fetch<T>(url: string, options?: FetchOptions): Promise<Response<T>> {
	if (!defaultClient)
		defaultClient = await createClient();

	return defaultClient.request({ ...options, url, method: options?.method ?? 'GET' });
}

export type { Client };
