#include <stdio.h>
#include <millennium.h>

#ifdef _WIN32
	#include <Windows.h>
#endif

int main(int argc, char **argv) {
	#ifdef _WIN32
    	SetConsoleOutputCP(CP_UTF8);
	#endif

	millennium::Builder builder;
	builder
		.invoke_handler([](void *, MillenniumInvoke *invoke) {
			printf("Event invoked: %s\n", millennium_invoke_message_command(invoke->message));
		}, NULL)
		.setup([](void *app) {
			printf("Hello, world! Callback from C++ ⚡\n");

			millennium::WindowBuilder windowBuilder(app, "second-window", "https://www.pyke.io");
			windowBuilder
				.title("Second window")
				.build();
		})
		.run();
	return 0;
}
