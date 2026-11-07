
language-name = Say "hi" \ bye

# $type (String) - a keyword
kind = Kind: { $type }

# $self (Number) - a keyword that can't be a raw identifier
count = { $self } items

# $args (String) - the name of the accessor's argument list
# $match (String) - another keyword
args = { $args } and { $match }

menu = Menu
    .tooltip = Tooltip for { $type }

# $loop (Element) - a link
# $segments (String) - the name of the accessor's segment array
nav = Go { $loop } to { $segments }
