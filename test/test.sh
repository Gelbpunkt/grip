#!/usr/bin/env bash
printf "Simple word matching (grip)...\n"
time=$(/usr/bin/time -f "%e" grip "abc" . >/dev/null)
printf "$time\n"
printf "Simple word matching (grep)...\n"
time=$(/usr/bin/time -f "%e" grep -R "abc" . >/dev/null)
printf "$time\n"

printf "Single simple regex matching (grip)...\n"
time=$(/usr/bin/time -f "%e" grip "[aA][bB][cC]" . >/dev/null)
printf "$time\n"
printf "Single simple regex matching (grep)...\n"
time=$(/usr/bin/time -f "%e" grep -Ri "[aA][bB][cC]" . >/dev/null)
printf "$time\n"
