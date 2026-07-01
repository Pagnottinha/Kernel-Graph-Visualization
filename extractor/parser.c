#include "parser.h"
#include "./db/db.h"
#include "utils.h"
#include <string.h>
#include <ctype.h>
#include <stdlib.h>
#include <limits.h>
#include <libgen.h>

static char project_root[PATH_MAX] = {0};

void set_project_root(const char *target_path) {
    if (realpath(target_path, project_root) == NULL) {
        strncpy(project_root, target_path, PATH_MAX - 1);
    }
    

    size_t len = strlen(project_root);
    if (len > 1 && project_root[len - 1] == '/') {
        project_root[len - 1] = '\0';
    }
}


const char* strip_project_root(const char *absolute_path) {
    size_t root_len = strlen(project_root);
    
    if (root_len > 0 && strncmp(absolute_path, project_root, root_len) == 0) {

        if (absolute_path[root_len] == '/') {
            return absolute_path + root_len + 1;
        }
    }
    return absolute_path; 
}

void parse_file_for_includes(const char *filepath, int source_id) {
    DEFER_FILE FILE *file = fopen(filepath, "r");
    if (!file) return;

    char filepath_copy[PATH_MAX];
    strncpy(filepath_copy, filepath, sizeof(filepath_copy));
    char *dir = dirname(filepath_copy);

    char line[512];
    while (fgets(line, sizeof(line), file)) {
        char *ptr = line;
        while (isspace((unsigned char)*ptr)) ptr++;

        if (strncmp(ptr, "#include", 8) == 0) {
            char target_include[256] = {0};
            char *start = strpbrk(ptr, "\"<");

            if (start) {
                char end_char = (*start == '"') ? '"' : '>';
                char *end = strchr(start + 1, end_char);
                if (end) {
                    strncpy(target_include, start + 1, end - start - 1);
                    
                    char full_path[PATH_MAX];
                    char resolved_path[PATH_MAX];

                    snprintf(full_path, sizeof(full_path), "%s/%s", dir, target_include);

                    if (realpath(full_path, resolved_path) != NULL) {

                        const char *final_target = target_include; 
                        const char *stripped = strip_project_root(resolved_path);
                        
                        if (stripped != resolved_path) {
                            final_target = stripped;
                        } 
                        
                        int target_id = db_get_or_create_file_id(final_target);
                        db_insert_edge(source_id, target_id);
                    } else {

                        const char *pure_filename = strrchr(target_include, '/');
                        pure_filename = pure_filename ? pure_filename + 1 : target_include;
                        int target_id = db_resolve_include_suffix(pure_filename);
                        
                        if (target_id != -1) {
                            db_insert_edge(source_id, target_id);
                        } else {
                            target_id = db_get_or_create_file_id(target_include);
                            db_insert_edge(source_id, target_id);
                        }
                    }
                }
            }
        }
    }
}
