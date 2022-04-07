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
		.invoke_handler([](MillenniumInvoke *invoke) {
			printf("Event invoked: %s\n", millennium_invoke_message_command(invoke->message));
		})
		.setup([](void *app) {
			printf("Hello, world! Callback from C++ ⚡\n");

			millennium::WindowBuilder windowBuilder(app, "second-window", "https://pyke.io/", true);
			windowBuilder
				.title("Second window")
				.build();
		})
		.run();
	return 0;
}
