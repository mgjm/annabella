.PHONY: example-hello-world build clean

example-hello-world: build
	make -C examples/hello-world

build:
	cmake -B build
	cmake --build build -j "$(nproc)"
	cmake --install build --prefix ~/.local

clean:
	rm -rvf build
	make -C examples/hello-world clean
