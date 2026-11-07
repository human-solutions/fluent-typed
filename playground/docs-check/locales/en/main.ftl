# Every message the README and docs/*.md examples use.
language-name = English
greeting = Welcome!

# $name (String) - the user's name
hello = Hello { $name }

unread-messages = { $count ->
    [one] You have one message
   *[other] You have { $count } messages
}

login = Log in
    .placeholder = Enter your email

# $enabled (Bool) - Whether the feature is enabled.
feature-status = { $enabled ->
    [true] Enabled
   *[false] Disabled
}

greeting-for = Hi { $who }

# $count (Number) - How many unread.
# $icon (Element) - An icon injected by the app.
# -privacy-link (Element) - Link text the app wraps in a link.
notice = { $icon } You have { $count } unread, see { -privacy-link }.
-privacy-link = our privacy policy
