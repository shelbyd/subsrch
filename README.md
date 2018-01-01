# subsrch

Find maximal or minimal subsets that pass the given command.

```bash
# Finds the smallest subset of two lines that match 'qu'.
echo -e "foo\nbar\nbaz\nqux\nquux" | subsrch minimal -- sh -c "(grep qu | wc -l | xargs -I _ test _ -ge 2) > /dev/null"
```
