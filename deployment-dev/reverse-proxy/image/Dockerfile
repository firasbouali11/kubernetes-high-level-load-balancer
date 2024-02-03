FROM nginx

RUN apt update

RUN mkdir -p /etc/nginx/ssl

RUN chmod 700 /etc/nginx/ssl

COPY init.sh /init.sh
COPY server /server

COPY nginx.conf /etc/nginx/nginx.conf

COPY lb.* /etc/nginx/ssl

CMD ["bash" ,"init.sh"]