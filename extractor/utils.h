#ifndef __UTILS_H 

#define __UTILS_H

#include <stdio.h>
#include <stdlib.h>
// ---------------------------------------------------------
// MACROS DE DEFER (Requer GCC ou Clang)
// ---------------------------------------------------------
#define DEFER(cleanup_fn) __attribute__((cleanup(cleanup_fn)))

// Funções de limpeza para os defers
static inline void cleanup_file(FILE **fp) {
    if (*fp) {
        fclose(*fp);
    }
}

static inline void cleanup_free(void *p) {
    void **ptr = (void **)p;
    if (*ptr) {
        free(*ptr);
    }
}

// Use estas macros ao declarar as variáveis
#define DEFER_FILE DEFER(cleanup_file)
#define DEFER_FREE DEFER(cleanup_free)

// ---------------------------------------------------------
// MACROS DE LOG E VISUALIZAÇÃO
// ---------------------------------------------------------
#define LOG_INFO(fmt, ...)  printf("[INFO] " fmt "\n", ##__VA_ARGS__)
#define LOG_ERR(fmt, ...)   fprintf(stderr, "[ERRO] %s:%d: " fmt "\n", __FILE__, __LINE__, ##__VA_ARGS__)
#define PANIC(fmt, ...)     do { LOG_ERR(fmt, ##__VA_ARGS__); exit(EXIT_FAILURE); } while(0)

#endif
