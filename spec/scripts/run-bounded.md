# Bounded command runner

`scripts/run-bounded.py --timeout SECONDS -- COMMAND ...` runs a development,
package installation, build, or verification command in a separate process group.
The timeout must be positive. Standard streams pass through unchanged. The runner
prints the child PID and deadline to stderr and returns the command's exit status.

On timeout it terminates the process group, allows five seconds for shutdown,
then kills remaining group members. Timeout returns 124. Ctrl-C performs the same
cleanup and returns 130. This runner does not manage pre-existing processes or
claim that a server's port has been released; callers verify that separately.
