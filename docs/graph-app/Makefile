NAME := obsidian_plugin
SRC := docker-compose.yml

DOCKER_COMPOSE_CMD := docker compose -f ${SRC} -p ${NAME}
TARGET_DEV := cat docker-compose.yml | sed 's/target: starter/target: builder/' | docker compose -f - -p ${NAME} up --build -d

all: down build up

build:
	${DOCKER_COMPOSE_CMD} build

up:
	${DOCKER_COMPOSE_CMD} up

frontend_dev:
	${TARGET_DEV} frontend
	docker attach obsidian_plugin-frontend-1

frontend:
	${DOCKER_COMPOSE_CMD} up --build frontend

down:
	${DOCKER_COMPOSE_CMD} down

clean:
	${DOCKER_COMPOSE_CMD} down -v
	@docker system prune -a --volumes


re : clean all

.PHONY : all clean fclean re