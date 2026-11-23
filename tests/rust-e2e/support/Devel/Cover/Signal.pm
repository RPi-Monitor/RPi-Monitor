package Devel::Cover::Signal;

use strict;
use warnings;
use POSIX ();

# Persist child-process coverage before SIGTERM's default action skips END blocks.
$SIG{TERM} = sub {
    Devel::Cover::report();
    $SIG{TERM} = 'DEFAULT';
    kill 'TERM', $$ or POSIX::_exit(143);
};

1;
