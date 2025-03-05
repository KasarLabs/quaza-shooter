#!/bin/bash

# Ensure that two arguments are provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <block_start> <block_end>"
    exit 1
fi

BLOCK_START=$1
BLOCK_END=$2
NODE_URL="http://localhost:8080/feeder_gateway/get_block"

TOTAL_TX=0
START_TIME=0
END_TIME=0
PREV_TIMESTAMP=0

echo "--------------------------------------"
echo "Block | Transactions | Time (s) | TPS"
echo "--------------------------------------"

# Loop through the specified block range
for ((BLOCK=$BLOCK_START; BLOCK<=$BLOCK_END; BLOCK++)); do
    # Fetch block data
    BLOCK_DATA=$(curl -s "$NODE_URL?blockNumber=$BLOCK")

    # Extract block timestamp
    TIMESTAMP=$(echo "$BLOCK_DATA" | jq -r '.timestamp')

    # Extract number of transactions
    TX_COUNT=$(echo "$BLOCK_DATA" | jq '.transactions | length')

    # Update tracking variables
    if [ "$BLOCK" -eq "$BLOCK_START" ]; then
        START_TIME=$TIMESTAMP
    fi

    if [ "$BLOCK" -eq "$BLOCK_END" ]; then
        END_TIME=$TIMESTAMP
    fi

    TOTAL_TX=$((TOTAL_TX + TX_COUNT))

    # Compute TPS for the current block
    if [ "$PREV_TIMESTAMP" -ne 0 ]; then
        TIME_DIFF=$((TIMESTAMP - PREV_TIMESTAMP))
    else
        TIME_DIFF=1 # Avoid division by zero
    fi

    BLOCK_TPS=$(echo "scale=2; $TX_COUNT / $TIME_DIFF" | bc)

    echo "$BLOCK  | $TX_COUNT          | $TIME_DIFF       | $BLOCK_TPS TPS"

    PREV_TIMESTAMP=$TIMESTAMP
done

# Compute overall TPS
DURATION=$((END_TIME - START_TIME))

if [ "$DURATION" -gt 0 ]; then
    TPS=$(echo "scale=2; $TOTAL_TX / $DURATION" | bc)
else
    TPS=0
fi

echo "--------------------------------------"
echo "Total transactions: $TOTAL_TX"
echo "Total duration: $DURATION seconds"
echo "Overall Transactions Per Second (TPS): $TPS"
echo "--------------------------------------"
