FROM postgres:16

RUN apt-get update && apt-get -y install git build-essential postgresql-server-dev-16 libcurl4-openssl-dev 

RUN git clone --branch v1.6.2 https://github.com/citusdata/pg_cron.git
RUN cd pg_cron && make && make install

# Install pgsql-http
RUN git clone https://github.com/pramsey/pgsql-http.git
RUN cd pgsql-http && make && make install

# Add pg_cron to shared_preload_libraries
RUN echo "shared_preload_libraries = 'pg_cron'" >> /usr/share/postgresql/postgresql.conf.sample