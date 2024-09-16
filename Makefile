NAME := scop42

SHADER_DIR := ./shaders

TOOL_SHADER := compile-shaders
TOOL_SHADER_DOCKER_IMAGE := ${NAME}/${TOOL_SHADER}

all: compile-shaders run

run:
	cargo run

compile-shaders: tool-shader-docker-build
	@echo "[$@] Compiling shaders..."
	@docker run --rm -it -v $(SHADER_DIR):/host $(TOOL_SHADER_DOCKER_IMAGE)
	@echo "[$@] Shaders compiled."

tool-shader-docker-build:
	@echo "[$@] Building image..."
	@docker build -t ${TOOL_SHADER_DOCKER_IMAGE} -f ./tools/${TOOL_SHADER}/Dockerfile ./tools/${TOOL_SHADER}
	@echo "[$@] Image builded."

.PHONY: all docker-build compile-shaders clean