class_name Toast extends PanelContainer

@onready var _title: Label = $VBoxContainer/Title/Label
@onready var _dismiss: Button = $VBoxContainer/Title/Dismiss
@onready var _scroll: ScrollContainer = $VBoxContainer/Body/ScrollContainer
@onready var _lines: Label = $VBoxContainer/Body/ScrollContainer/Lines

func _ready() -> void:
	_dismiss.pressed.connect(hide)

func report(title: String, lines: Array[String]) -> void:
	if lines.is_empty():
		hide()
		return

	_title.text = title
	_lines.text = "- " + "\n- ".join(lines)
	_scroll.scroll_vertical = 0
	
	show()
