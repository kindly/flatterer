FROM quay.io/pypa/manylinux2014_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp312-cp312/bin/:/opt/python/cp311-cp311/bin/:/opt/python/cp310-cp310/bin/:/opt/python/cp309-cp309/bin/:/opt/python/cp308-cp308/bin/:/opt/python/cp307-cp307/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && python3 -m pip install --no-cache-dir cffi maturin \
    && ln -s $(which maturin) /usr/bin/maturin \
    && mkdir /io

RUN yum install centos-release-scl -y \
    && sed -i \
            -e 's/^mirrorlist/#mirrorlist/' \
            -e 's/^#baseurl/baseurl/' \
            -e 's/mirror\.centos\.org/vault.centos.org/' \
            /etc/yum.repos.d/CentOS-SCLo-scl-rh.repo \
    && yum-config-manager --enable rhel-server-rhscl-7-rpms \
    && yum install llvm-toolset-7.0 -y \
    && yum install openssl-devel -y
WORKDIR /io

COPY docker-entrypoint.sh /usr/bin/entrypoint.sh
RUN chmod +x /usr/bin/entrypoint.sh
ENTRYPOINT [ "/usr/bin/entrypoint.sh" ]
