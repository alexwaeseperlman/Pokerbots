x=$(./tim)
while true; do
	echo $(($(./tim) - $x)) >>time.log
done
