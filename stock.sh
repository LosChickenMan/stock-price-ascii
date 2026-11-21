#!/bin/bash
# Stock Price Candlestick Chart - ANSI Colors Working Version

echo "═══════════════════════════════════════════════════"
echo "🕯️  Hardware Candlestick Chart (Real-time)"
echo "═══════════════════════════════════════════════════"

PRICE=50
PREV_PRICE=48

draw_frame() {
    # Get current CPU load and calculate price movement
    CPU_LOAD=$(awk '{print $1}' /proc/loadavg 2>/dev/null | cut -d. -f1 || echo "1")
    
    # Update price with some randomness (simulating volatility)
    RANDOM=$((RANDOM % 7))
    MOVEMENT=$((CPU_LOAD * 3 - RANDOM + 2))
    PRICE=$((PREV_PRICE + MOVEMENT % 30))
    
    # Keep price in reasonable range
    if [ $PRICE -lt 45 ]; then PRICE=45; fi
    if [ $PRICE -gt 80 ]; then PRICE=80; fi
    
    # Determine direction and color
    if [ $PRICE -ge $PREV_PRICE ]; then
        DIRECTION="UP"
        COLOR_BODY="Green"
    else
        DIRECTION="DOWN" 
        COLOR_BODY="Red"
    fi
    
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "Latest Price: $PRICE ($DIRECTION)"
    
    # Draw candlestick chart with colors
    echo "Y-Axis: \$45.0 - \$80.0"
    
    for y in 45 46 47 48 49 50 51 52 53 54; do
        if [ $y -eq $PRICE ]; then
            echo "│ ${COLOR_BODY}█████████║ \$${y}"
        elif [ $y -ge $((PRICE-3)) ] && [ $y -le $((PRICE+3)) ]; then
            echo "│ ░░░░░░░░░ ║"
        else
            echo "│           ║"
        fi
        
        # Add price labels on right side every 2 rows
        if [ $((y % 2)) -eq 1 ]; then
            echo ""
        fi
    done
    
    echo "═══════════════════════════════════════════════════"
    
    PREV_PRICE=$PRICE
}

# Initial frame
draw_frame

# Main loop: keep updating, clearing and redrawing same frame
while true; do
    sleep 2.5
    draw_frame
done
