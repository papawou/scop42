FROM archlinux

RUN pacman -Syu shaderc --noconfirm

COPY ./compile_shaders.sh /tmp/compile_shaders.sh

VOLUME /host
WORKDIR /host

CMD /tmp/compile_shaders.sh