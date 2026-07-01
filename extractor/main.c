#define _XOPEN_SOURCE 500
#include <ftw.h>
#include <string.h>
#include "utils.h"
#include "./db/db.h"
#include "parser.h"

static int is_source_file(const char *fpath) {
    const char *ext = strrchr(fpath, '.');
    return ext && (strcmp(ext, ".c") == 0 || strcmp(ext, ".h") == 0 ||
                   strcmp(ext, ".cpp") == 0 || strcmp(ext, ".hpp") == 0 ||
                   strcmp(ext, ".cc") == 0 || strcmp(ext, ".hh") == 0);
}

// Register every source file so include resolution can find any file regardless of the directory walk
// order. Without this, an include whose target has not been walked yet falls through to a phantom
// node.
int register_path(const char *fpath, const struct stat *sb, int typeflag, struct FTW *ftwbuf) {
    (void)sb;
    (void)ftwbuf;

    if (typeflag != FTW_F) return 0;

    if (is_source_file(fpath)) {
        const char *relative_source = strip_project_root(fpath);
        db_get_or_create_file_id(relative_source);
    }
    return 0;
}

// Parse each file's #include directives against the fully populated file index.
int parse_path(const char *fpath, const struct stat *sb, int typeflag, struct FTW *ftwbuf) {
    (void)sb;
    (void)ftwbuf;

    if (typeflag != FTW_F) return 0;

    if (is_source_file(fpath)) {
        const char *relative_source = strip_project_root(fpath);
        int source_id = db_get_or_create_file_id(relative_source);
        parse_file_for_includes(fpath, source_id);
    }
    return 0;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        PANIC("Uso: %s <caminho_do_kernel>", argv[0]);
    }

    db_init("kernel_graph.db");

    LOG_INFO("Analisando diretório: %s", argv[1]);
    set_project_root(argv[1]);

    // Two passes: register all files first, then resolve includes.
    if (nftw(argv[1], register_path, 15, FTW_PHYS) == -1) {
        PANIC("Erro ao percorrer diretórios.");
    }
    if (nftw(argv[1], parse_path, 15, FTW_PHYS) == -1) {
        PANIC("Erro ao percorrer diretórios.");
    }

    db_close();
    LOG_INFO("Extração concluída com sucesso!");
    return 0;
}
