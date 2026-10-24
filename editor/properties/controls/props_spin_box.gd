class_name PropsSpinBox extends SpinBox

func _ready() -> void:
	select_all_on_focus = true

	var edit := get_line_edit()

	edit.caret_blink = false
	edit.add_theme_color_override(&"caret_color", Color.TRANSPARENT)
	edit.add_theme_constant_override(&"minimum_character_width", 0)

	edit.alignment = HORIZONTAL_ALIGNMENT_LEFT

	edit.draw.connect(func() -> void:
		_want(edit)
		_rewind(edit)
		_caret(edit))

	edit.focus_exited.connect(func():
		if edit.text.strip_edges().is_empty():
			set_deferred(&"value", 0.0))

func editing() -> bool:
	return get_line_edit().has_focus()

func _want(edit: LineEdit) -> void:
	var wish := ceilf(edit.get_theme_stylebox(&"normal").get_minimum_size().x
		+ Palette.advance(edit, edit.text))

	if get_meta(&"wish", -1.0) == wish:
		return

	set_meta(&"wish", wish)

	var rows := get_parent()

	if rows is Container:
		rows.queue_sort()

func _rewind(edit: LineEdit) -> void:
	if edit.get_scroll_offset() == 0.0:
		return

	var whole := (edit.has_selection() and edit.get_selection_from_column() == 0
		and edit.get_selection_to_column() == edit.text.length())

	if edit.has_focus() and edit.is_editing() and not whole:
		return

	edit.caret_column = 0

func _caret(edit: LineEdit) -> void:
	if not edit.has_focus() or not edit.is_editing():
		return

	var font := edit.get_theme_font(&"font")
	var size_ := edit.get_theme_font_size(&"font_size")
	var box := edit.get_theme_stylebox(&"normal")
	var under := edit.text.substr(edit.caret_column, 1)

	var at := (box.content_margin_left + edit.get_scroll_offset()
		+ Palette.advance(edit, edit.text.left(edit.caret_column)))

	var height := font.get_height(size_)
	var top := box.content_margin_top

	edit.draw_rect(Rect2(at, top, Palette.advance(edit, under) if under != ""
		else Palette.CARET, height), Palette.FG)

	if under != "":
		edit.draw_string(font, Vector2(at, top + font.get_ascent(size_)), under,
			HORIZONTAL_ALIGNMENT_LEFT, -1, size_, Palette.INK)
