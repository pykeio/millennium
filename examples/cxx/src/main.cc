#include <stdio.h>
#include <millennium.h>

#ifdef _WIN32
	#include <Windows.h>
#endif

int main(int argc, char **argv) {
	#ifdef _WIN32
    	SetConsoleOutputCP(CP_UTF8);
	#endif

	MillenniumBuilder builder = millennium_builder_new();
	millennium_builder_invoke_handler(builder, [](void *, MillenniumInvoke *invoke) {
		printf("Event invoked: %s\n", millennium_invoke_message_command(invoke->message));
	}, NULL);
	millennium_builder_setup(builder, [](void *, void *app) {
		printf("Hello, world! Callback from C++ ⚡\n");
	}, NULL);
	millennium_builder_run(builder);
	return 0;
}
