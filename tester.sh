signature="U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgJabVu8/+5jRPCocvPKKQIjYh6LKEpM94PtELy49zrOsAAAAEZmlsZQAAAAAAAAAGc2hhNTEyAAAAUwAAAAtzc2gtZWQyNTUxOQAAAEChvyD3We1EY6ahq6ErgRjtiv87EAtsFIaCjOwDnzs0cED6+IJTf7SU3ZA1t/WkqcY8TfVFgcdTSUmJmwcbAL4G"
private_key="b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZWQyNTUxOQAAACAlptW7z/7mNE8Khy88opAiNiHosoSkz3g+0QvLj3Os6wAAAKg9NyEfPTchHwAAAAtzc2gtZWQyNTUxOQAAACAlptW7z/7mNE8Khy88opAiNiHosoSkz3g+0QvLj3Os6wAAAECDw5AbmnKIuYlBVpIv259mHyk8d2Uk+WNRLWsx/b07kyWm1bvP/uY0TwqHLzyikCI2IeiyhKTPeD7RC8uPc6zrAAAAImdhYnJpZWxAZ2FicmllbC1UaGlua1BhZC1MMTUtR2VuLTEBAgM="

	#--data '{"id":"abc", "public_key": '$public_key'}' \
dna_sequence="TACG"
echo "Add public key:"
result=$(curl --header "Content-Type: application/json" \
	--request POST \
	--data '{"id":"abc", "public_key": "AAAAC3NzaC1lZDI1NTE5AAAAICWm1bvP/uY0TwqHLzyikCI2IeiyhKTPeD7RC8uPc6zr"}' \
	127.0.0.1:8082/insert_public_key)
echo "result: "$result

echo "Add dna: "
result=$(curl --header "Content-Type: application/json" \
	--request POST \
	--data '{"id":'$result', "dna_sequence": "'$dna_sequence'", "signature": "'$signature'"}' \
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
