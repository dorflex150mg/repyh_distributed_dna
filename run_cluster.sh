cmd='rm dna.db; FILENAME="conf/ips0.json" cargo run'
for i in {0..4}; do
#	docker run --net="host" -d ubuntu /bin/bash -c 'rm dna.db; FILENAME="conf/ips{i}.json" cargo run';
	rm dna.db 
	DATABASE="var/dna$i.db" FILENAME="conf/ips$i.json" cargo run &
done
#docker run --net="host" -it ubuntu /bin/bash -c 'rm dna.db; FILENAME="conf/ips4.json" cargo run';
