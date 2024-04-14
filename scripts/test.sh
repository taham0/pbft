killall {node} &> /dev/null
rm -rf /tmp/*.db &> /dev/null
vals=(27000 27100 27200 27300)
# vals=(27000 27100 27200 27300 27400 27500 27600 27700 27800 27900 28000 28100 28200 28300 28400 28500)

rand=$(shuf -i 1000-150000000 -n 1)
TESTDIR=${TESTDIR:="testdata/hyb_4"}
TYPE=${TYPE:="release"}

# Run the syncer now
./target/$TYPE/node \
    --config $TESTDIR/nodes-0.json \
    --ip ip_file \
    --protocol sync \
    --input 100 \
    --syncer $1 \
    --byzantine false > logs/syncer.log &

# for((i=0;i<11;i++)); do
# ./target/$TYPE/node \
#     --config $TESTDIR/nodes-$i.json \
#     --ip ip_file \
#     --protocol pbft \
#     --input ${vals[$i]} \
#     --syncer $1 \
#     --byzantine false > logs/$i.log &
# done

# for((i=11;i<16;i++)); do
# ./target/$TYPE/node \
#     --config $TESTDIR/nodes-$i.json \
#     --ip ip_file \
#     --protocol pbft \
#     --input ${vals[$i]} \
#     --syncer $1 \
#     --byzantine true > logs/$i.log &
# done

for((i=0;i<3;i++)); do
./target/$TYPE/node \
    --config $TESTDIR/nodes-$i.json \
    --ip ip_file \
    --protocol pbft \
    --input ${vals[$i]} \
    --syncer $1 \
    --byzantine false > logs/$i.log &
done

for((i=3;i<4;i++)); do
./target/$TYPE/node \
    --config $TESTDIR/nodes-$i.json \
    --ip ip_file \
    --protocol pbft \
    --input ${vals[$i]} \
    --syncer $1 \
    --byzantine true > logs/$i.log &
done

# sudo lsof -ti:7000-7015 | xargs kill -9
# ./scripts/test.sh testdata/hyb_4/syncer Hi false