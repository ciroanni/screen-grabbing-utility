# PDS project 2023
## Rust code for a screen grabbing utility capable of acquiring what is currently shown in a display, post-process it and make it available in one or more formats.

:bangbang:v1.1.0: Fixed some bugs and improving ui... 


v1.0.9: You can now have a delay.


v1.0.8: Correction of the bug of the screenshot in the drag and drop feature.

v1.0.7: Animation for the click and drag motion added and screen issue fixed.

v1.0.6: You can now have a custom area to select with a click and drag motion.


v1.0.5: You can now take screen with a customizable shortcut (Feature 4).


v1.0.4: Controller to take screen using non-customizable shortcut (ALT + S). 


v1.0.3: Better UI with a dropdwon select list for the output format. (Adding [this library](https://github.com/linebender/druid-widget-nursery). See [Cargo.toml](/project/Cargo.toml))


v1.0.2: Radio buttons for output format added. (Feature 5)


v1.0.1: Makes a screenshot of the entire main display. Feature 1 & 2 & 5 partially implemented.

### Future features:

1. Platform Support: The utility should be compatible with multiple desktop operating systems, including Windows, macOS, and Linux.
2. User Interface (UI): The utility should have an intuitive and user-friendly interface that allows users to easily navigate through the application's features.
3. Selection Options: The utility should allow the user to restrict the grabbed image to a custom area selected with a click and drag motion. The selected area may be further adjusted with subsequent interactions.
4. Hotkey Support: The utility should support customizable hotkeys for quick screen grabbing. Users should be able to set up their preferred shortcut keys.
5. Output Format: The utility should support multiple output formats including .png, .jpg, .gif. It should also support copying the screen grab to the clipboard.

### Bonus features:

6. Annotation Tools: The utility should have built-in annotation tools like shapes, arrows, text, and a color picker for highlighting or redacting parts of the screen grab.
7. Delay Timer: The utility should support a delay timer function, allowing users to set up a screen grab after a specified delay.
8. Save Options: The utility should allow users to specify the default save location for screen grabs. It should also support automatic saving with predefined naming conventions.
9. Multi-monitor Support: The utility should be able to recognize and handle multiple monitors independently, allowing users to grab screens from any of the connected displays.
