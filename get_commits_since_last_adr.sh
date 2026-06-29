#!/bin/bash
# Get the date of the last ADR
LAST_ADR_DATE=$(grep -h "Date:" docs/adr/*.md | sort -r | head -n 1 | awk '{print $2}')
echo "Last ADR Date: $LAST_ADR_DATE"

# Get commits since that date
git log --since="$LAST_ADR_DATE" --oneline
