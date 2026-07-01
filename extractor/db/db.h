#ifndef __DB_H

#define __DB_H

void db_init(const char *db_name);
int db_get_or_create_file_id(const char *filepath);
void db_insert_edge(int source_id, int target_id);
int db_resolve_include_suffix(const char *include_path);
void db_close(void);

#endif
