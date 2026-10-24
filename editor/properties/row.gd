@tool
class_name Row extends Container

const DOT := 1
const STEP := 3

func _ready() -> void:
	theme_type_variation = &"Row"

	var label := _cell(0) as Label

	if label != null:
		label.clip_text = true
		label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS

func _get_minimum_size() -> Vector2:
	var name_cell := _cell(0)
	var value_cell := _cell(1)

	if name_cell == null or value_cell == null:
		return Vector2.ZERO

	var name_size := name_cell.get_combined_minimum_size()
	var value_size := value_cell.get_combined_minimum_size()

	return Vector2(name_size.x + get_theme_constant(&"h_separation") + value_size.x,
		maxf(name_size.y, value_size.y))

func _notification(what: int) -> void:
	if what != NOTIFICATION_SORT_CHILDREN:
		return

	var name_cell := _cell(0)
	var value_cell := _cell(1)

	if name_cell == null or value_cell == null:
		return

	var gap := get_theme_constant(&"h_separation")
	var name_size := name_cell.get_combined_minimum_size()
	var value_size := value_cell.get_combined_minimum_size()
	var height := maxf(name_size.y, value_size.y)

	var slack := maxf(size.x - gap - _text_width(name_cell), value_size.x)
	var wish = value_cell.get_meta(&"wish", value_size.x)
	var value_width := minf(clampf(wish, value_size.x, slack), size.x)

	fit_child_in_rect(name_cell,
		Rect2(0, 0, maxf(size.x - gap - value_width, 0.0), height))
	fit_child_in_rect(value_cell,
		Rect2(size.x - value_width, 0, value_width, height))

	queue_redraw()

func _draw() -> void:
	var name_cell := _cell(0)
	var value_cell := _cell(1)

	if name_cell == null or value_cell == null:
		return

	var color := get_theme_color(&"leader")
	var gap := get_theme_constant(&"h_separation")

	var at := ceilf((_text_width(name_cell) + gap) / STEP) * STEP
	var until := value_cell.position.x - gap
	var y := floorf(name_cell.position.y + name_cell.size.y * 0.5)

	while at + DOT <= until:
		draw_rect(Rect2(at, y, DOT, DOT), color)
		at += STEP

func value() -> Control:
	return _cell(1)

func _cell(at: int) -> Control:
	if get_child_count() <= at:
		return null

	return get_child(at) as Control

func _text_width(cell: Control) -> float:
	var label := cell as Label

	if label == null:
		return cell.get_minimum_size().x

	return Palette.advance(label, label.text)
