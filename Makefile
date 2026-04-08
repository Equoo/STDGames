.PHONY: release dev

setup:
	./setup.sh

frontdev:
	docker run --rm -it -p 5173:5173 -v $(shell pwd):/app -w /app node:20 npm run dev -- --host 0.0.0.0

frontbuild:
	docker run --rm -it -p 5173:5173 -v $(shell pwd):/app -w /app node:20 npm run build

dev:
	@mkdir -p /tmp/stdgame_target
	@docker build -t stddev:latest -f Dockerfile.dev .
	@env | grep -v PATH > .env.docker
	@docker run -it --rm \
		--ipc=host \
		--network host \
		-u $(id -u):$(id -g) \
		-v $(PWD)/:/app \
		-v /tmp/stdgame_target:/app/src-tauri/target \
		-v /sgoinfre:/sgoinfre \
		-v /goinfre:/goinfre \
		-v /run/user/$(shell id -u):/run/user/$(shell id -u) \
		-v /run/user/$(shell id -u)/at-spi/bus_0:/run/user/0/at-spi/bus_0 \
		-v /run/user/$(shell id -u)/pulse:/run/user/0/pulse \
		-v /tmp:/tmp \
		-v /tmp/.X11-unix:/tmp/.X11-unix \
		-v /dev/dri:/dev/dri \
		-v /home/$(USER):/home/$(USER) \
		--env-file .env.docker \
		-e GDK_BACKEND=x11 \
		-e GDK_SCALE=1 \
		-e GTK_MODULES='' \
		-e NO_AT_BRIDGE=1 \
		-e QT_X11_NO_MITSHM=1 \
		-e GDK_USE_X11=1 \
		-e GDK_DISABLE_MITSHM=1 \
		-e DISPLAY=$(DISPLAY) \
		stddev

release:
	@docker build -t stdbuild:latest -f Dockerfile.release .
	@docker run -it --rm \
		-v $(PWD)/:/app \
		-v $(PWD)/src-tauri/target:/app/src-tauri/target \
		stdbuild
