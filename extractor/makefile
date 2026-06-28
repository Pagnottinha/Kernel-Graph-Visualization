path ?= ./

build:
	gcc main.c ./db/db.c parser.c -lsqlite3 -O3 -o extractor

deploy:
	mv kernel_graph.db ../ui_graph

run:
	rm -f kernel_graph.db
	./extractor ${path}
