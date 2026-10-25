#!/usr/bin/perl
#
# Unit-level entry point for the pytest harness: compile the packages of
# rpimonitord without running its main section, evaluate a Perl expression
# against them and print its result as JSON on STDOUT.
#
# Usage: perl_unit.pl <rpimonitord> <expression> [args...]
#   <expression> sees [args...] in @ARGV.
#
use strict;
use JSON;

my $usage = "Usage: $0 <rpimonitord> <expression> [args...]\n";
my $binary = shift or die $usage;
my $code = shift or die $usage;

# $/ is localised in a block: left undefined, it would break every
# line-by-line read done by the code under test.
my $source = do {
  open(my $fh, '<', $binary) or die "Can't read $binary: $!\n";
  local $/;
  <$fh>;
};

# Everything from `package main;` on parses @ARGV and starts the daemon.
$source =~ s/^package main;.*//ms or die "'package main;' not found in $binary\n";

# #line keeps error messages and Devel::Cover pointing at the real file.
eval "#line 1 \"$binary\"\n$source; 1" or die $@;

my $result = eval $code;
die $@ if $@;
print JSON->new->canonical->allow_nonref->encode($result), "\n";
