.PHONY: runtime transpiler

runtime:
	cmake -B build
	cmake --build build -j "$(nproc)" -t annabella-rt
	cmake --install build --prefix ~/.local

transpiler:
	cmake -B build
	cmake --build build -j "$(nproc)" -t annabella
	cmake --install build --prefix ~/.local

