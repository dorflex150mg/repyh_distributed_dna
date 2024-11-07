echo "Add dna: "
result=$(curl --header "Content-Type: application/json" \
	--request POST \
	--data '{"id":"abc", "dna_sequence":"TACG"}' \
	127.0.0.1:8082/insert_dna_sequence)
echo "result: "$result
data='{"id":'$result'}'
echo "Data: "$data
echo " "
echo "Get dna: "
curl --header "Content-Type: application/json" \
	--request GET \
	--data $data \
	127.0.0.1:8082/dna
echo " "
echo "Patch dna: "
result=$(curl --header "Content-Type: application/json" \
	--request POST \
	--data '{"id":'$result', "dna_sequence":"TCCG"}' \
	127.0.0.1:8082/insert_dna_sequence)
echo "result: "$result
echo "Get patched dna: "
curl --header "Content-Type: application/json" \
	--request GET \
	--data $data \
	127.0.0.1:8082/dna
echo " "
