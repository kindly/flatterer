FROM quay.io/pypa/manylinux2014_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp35-cp35m/bin/:/opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:/opt/python/cp38-cp38/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && python3 -m pip install --no-cache-dir cffi maturin \
    && ln -s $(which maturin) /usr/bin/maturin \
    && mkdir /io

RUN yum install centos-release-scl -y \
    && yum-config-manager --enable rhel-server-rhscl-7-rpms \
    && yum install llvm-toolset-7.0 -y

WORKDIR /io

COPY docker-entrypoint.sh /usr/bin/entrypoint.sh
RUN chmod +x /usr/bin/entrypoint.sh
ENTRYPOINT [ "/usr/bin/entrypoint.sh" ]
