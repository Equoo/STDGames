#include "fetch.h"
#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <netdb.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <fcntl.h>

void fetch_build_url(const char *remote_path, char *out_url, size_t out_len)
{
    const httpfs_config_t *cfg = httpfs_get_config();
    const char *prefix = cfg->prefix;
    size_t plen = strlen(prefix);
    const char *rel = (strncmp(remote_path, prefix, plen) == 0)
                      ? remote_path + plen : remote_path;
    snprintf(out_url, out_len, "%s/%s", cfg->base_url, rel);
}

/* Parse "http://host:port/path" — retourne 0 si ok */
static int parse_url(const char *url,
                     char *host, size_t hlen,
                     char *port, size_t plen,
                     char *path, size_t pathlen)
{
    /* saute "http://" */
    const char *p = url;
    if (strncmp(p, "http://", 7) == 0) p += 7;

    /* host:port */
    const char *slash = strchr(p, '/');
    const char *colon = strchr(p, ':');

    if (colon && (!slash || colon < slash)) {
        size_t hsize = (size_t)(colon - p);
        if (hsize >= hlen) return -1;
        strncpy(host, p, hsize); host[hsize] = '\0';
        size_t psize = slash ? (size_t)(slash - colon - 1) : strlen(colon + 1);
        if (psize >= plen) return -1;
        strncpy(port, colon + 1, psize); port[psize] = '\0';
    } else {
        size_t hsize = slash ? (size_t)(slash - p) : strlen(p);
        if (hsize >= hlen) return -1;
        strncpy(host, p, hsize); host[hsize] = '\0';
        strncpy(port, "80", plen);
    }

    strncpy(path, slash ? slash : "/", pathlen);
    return 0;
}

int fetch_url(const char *url, const char *dest_tmp)
{
    const httpfs_config_t *cfg = httpfs_get_config();

    char host[256], port[16], path[4096];
    if (parse_url(url, host, sizeof(host), port, sizeof(port),
                  path, sizeof(path)) < 0) {
        fprintf(stderr, "[httpfs] bad url: %s\n", url);
        return -1;
    }

    /* Résolution DNS */
    struct addrinfo hints = {0}, *res = NULL;
    hints.ai_family   = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    if (getaddrinfo(host, port, &hints, &res) != 0) {
        fprintf(stderr, "[httpfs] getaddrinfo failed: %s:%s\n", host, port);
        return -1;
    }

    int sock = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (sock < 0) { freeaddrinfo(res); return -1; }
    if (connect(sock, res->ai_addr, res->ai_addrlen) < 0) {
        freeaddrinfo(res); close(sock); return -1;
    }
    freeaddrinfo(res);

    /* Requête HTTP/1.1 */
    char req[8192];
    int reqlen = snprintf(req, sizeof(req),
        "GET %s HTTP/1.1\r\n"
        "Host: %s:%s\r\n"
        "Connection: close\r\n"
        "User-Agent: httpfs_preload/1.0\r\n"
        "\r\n",
        path, host, port);
    if (send(sock, req, reqlen, 0) < 0) { close(sock); return -1; }

    if (cfg->verbose)
        fprintf(stderr, "[httpfs] fetch: GET %s\n", url);

    /* Lecture réponse — on cherche \r\n\r\n pour séparer header/body */
    FILE *fp = fopen(dest_tmp, "wb");
    if (!fp) { close(sock); return -1; }

    char buf[65536];
    ssize_t n;
    int header_done = 0;
    int status = 0;
    char leftover[65536];
    size_t leftover_len = 0;

    while ((n = recv(sock, buf, sizeof(buf), 0)) > 0) {
        if (!header_done) {
            /* Cherche fin des headers dans ce chunk + leftover */
            char combined[131072];
            size_t clen = leftover_len + (size_t)n;
            if (clen > sizeof(combined)) clen = sizeof(combined);
            memcpy(combined, leftover, leftover_len);
            memcpy(combined + leftover_len, buf, (size_t)n);

            char *sep = memmem(combined, clen, "\r\n\r\n", 4);
            if (sep) {
                /* Extrait le status code */
                sscanf(combined, "HTTP/%*s %d", &status);
                header_done = 1;
                char *body = sep + 4;
                size_t body_len = clen - (size_t)(body - combined);
                if (body_len > 0)
                    fwrite(body, 1, body_len, fp);
            } else {
                /* Pas encore la fin des headers */
                leftover_len = clen > sizeof(leftover) ? sizeof(leftover) : clen;
                memcpy(leftover, combined, leftover_len);
            }
        } else {
            fwrite(buf, 1, (size_t)n, fp);
        }
    }

    fclose(fp);
    close(sock);

    if (status != 200) {
        fprintf(stderr, "[httpfs] HTTP %d for %s\n", status, url);
        unlink(dest_tmp);
        return -1;
    }
    return 0;
}
