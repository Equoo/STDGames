.PHONY: release dev

release:
	@docker build -t stdbuild:latest -f Dockerfile.release .
	@docker run -it --rm \
		-v $(PWD)/:/app \
		-v $(PWD)/src-tauri/target:/app/src-tauri/target \
		-v $(HOME)/.cargo:/usr/local/cargo \
		-v $(HOME)/.rustup:/usr/local/rustup \
		stdbuild

dev:
	@xhost +local:docker
	@docker build -t stddev:latest -f Dockerfile.dev .
	@docker run --cap-add=SYS_ADMIN --cap-add=SYS_CHROOT --privileged -it --rm \
		--ipc=host \
		-v $(PWD)/:/app \
		-v /sgoinfre:/sgoinfre \
		-v /goinfre:/goinfre \
		-v /tmp:/tmp \
		-v $(PWD)/src-tauri/target:/app/src-tauri/target \
		-v /tmp/.X11-unix:/tmp/.X11-unix \
		-v /dev/dri:/dev/dri \
		-v /run/user/$(shell id -u)/at-spi/bus_0:/run/user/0/at-spi/bus_0 \
		-v /run/user/$(shell id -u)/pulse:/run/user/0/pulse \
		-e USER=$(USER) \
		-e GDK_BACKEND=x11 \
		-e GDK_SCALE=1 \
		-e GTK_MODULES='' \
		-e NO_AT_BRIDGE=1 \
		-e QT_X11_NO_MITSHM=1 \
		-e GDK_USE_X11=1 \
		-e GDK_DISABLE_MITSHM=1 \
		-e DISPLAY=$(DISPLAY) \
		stddev

