FROM quay.io/pypa/manylinux_2_28_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp313-cp313/bin/:/opt/python/cp312-cp312/bin/:/opt/python/cp311-cp311/bin/:/opt/python/cp310-cp310/bin/:/opt/python/cp309-cp309/bin/:/opt/python/cp308-cp308/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && python3 -m pip install --no-cache-dir cffi maturin \
    && ln -s $(which maturin) /usr/bin/maturin \
    && mkdir /io

RUN yum install llvm-toolset -y \
    && yum install openssl-devel -y
WORKDIR /io

COPY docker-entrypoint.sh /usr/bin/entrypoint.sh
RUN chmod +x /usr/bin/entrypoint.sh
ENTRYPOINT [ "/usr/bin/entrypoint.sh" ]
