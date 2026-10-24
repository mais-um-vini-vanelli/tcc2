class_name PropsButton extends Button

const ELLIPSIS := "…"

signal path_selected(path: String)

var path: String:
	get:
		return _path
	set(value):
		_path = value
		_hold()

var _path := ""
var _file := ""
var _dialog: FileDialog

func _ready() -> void:
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	theme_type_variation = &"FieldButton"
	alignment = HORIZONTAL_ALIGNMENT_RIGHT
	clip_text = true

	for child in get_children():
		if child is FileDialog:
			_dialog = child

	if _dialog == null:
		push_error("%s carries no FileDialog" % name)
		return

	resized.connect(_fit)

	pressed.connect(func():
		if not _path.is_empty():
			_dialog.current_path = _path

		_dialog.popup_centered_ratio(0.6))

	_dialog.file_selected.connect(func(selected: String):
		path = selected
		path_selected.emit(selected))

	_hold()

func _hold() -> void:
	if not is_node_ready():
		return

	_file = _path.get_file() if not _path.is_empty() else "(none)"

	tooltip_text = _path
	set_meta(&"wish", get_theme_stylebox(&"normal").get_minimum_size().x
		+ Palette.advance(self, _file))

	var rows := get_parent()

	if rows is Container:
		rows.queue_sort()

	_fit()

func _fit() -> void:
	var room := size.x - get_theme_stylebox(&"normal").get_minimum_size().x
	var shown := _file

	if room > 0.0 and Palette.advance(self, _file) > room:
		var kept := _file

		while not kept.is_empty() and Palette.advance(self, ELLIPSIS + kept) > room:
			kept = kept.substr(1)

		shown = ELLIPSIS + kept

	if text != shown:
		text = shown
