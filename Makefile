NAME := scop42

SHADER_DIR := ./shaders

TOOL_SHADER := compile-shaders
TOOL_SHADER_DOCKER_IMAGE := ${NAME}/${TOOL_SHADER}

tool-shader-docker-build:
	@echo "${TOOL_SHADER}: docker: building image..."
	docker build -t ${TOOL_SHADER_DOCKER_IMAGE} -f ./tools/${TOOL_SHADER}/Dockerfile ./tools/${TOOL_SHADER}
	@echo "${TOOL_SHADER}: docker: image builded."

compile-shaders: tool-shader-docker-build
	@echo "${TOOL_SHADER}: docker: compiling shaders..."
	docker run --rm -it -v $(SHADER_DIR):/host $(TOOL_SHADER_DOCKER_IMAGE)
	@echo "${TOOL_SHADER}: docker: shaders compiled."

.PHONY: all docker-build compile-shaders clean