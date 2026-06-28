#ifndef __PARSER_H

#define __PARSER_H

void set_project_root(const char *target_path);
const char* strip_project_root(const char *absolute_path);
void parse_file_for_includes(const char *filepath, int source_id);

#endif
