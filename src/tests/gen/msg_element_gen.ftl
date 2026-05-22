
# $num (Number) - How many.
# $provider (String) - The calendar provider.
# $icon (Element) - A UI element injected by the app.
# -privacy-link (Element) - Translatable text the app wraps.
calendar-sync-description =
    Sync calendar { $num } using { $provider ->
        [google] Google Calendar
       *[other] external calendar
    } { $icon } feed, see { -privacy-link } for more information.
-privacy-link = our privacy policy
