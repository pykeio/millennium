#include "millennium.h"

int main(int argc, char **argv) {
	MillenniumBuilder *builder = millennium_builder_new();
	millennium_builder_run(builder);
	return 0;
}
