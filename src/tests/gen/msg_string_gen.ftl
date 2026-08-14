

# $name (String) - The name.
greeting = Hi { $name }

# $enabled (Bool) - Whether the feature is enabled.
feature-status = { $enabled ->
    [true] Enabled
   *[false] Disabled
}

