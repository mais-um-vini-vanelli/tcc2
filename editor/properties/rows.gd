@tool
class_name Rows extends Container

const DOT := 1
const STEP := 3

func _ready() -> void:
	theme_type_variation = &"Rows"

	for child in get_children():
		var label := child as Label

		if label == null:
			continue

		label.clip_text = true
		label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS

func _get_minimum_size() -> Vector2:
	var gap := get_theme_constant(&"h_separation")
	var lead := get_theme_constant(&"v_separation")
	var total := Vector2.ZERO
	var rows := 0

	for pair in _pairs():
		var name_size: Vector2 = pair[0].get_combined_minimum_size()
		var value_size: Vector2 = pair[1].get_combined_minimum_size()

		total.x = maxf(total.x, name_size.x + gap + value_size.x)
		total.y += maxf(name_size.y, value_size.y)
		rows += 1

	total.y += lead * maxi(rows - 1, 0)

	return total

func _notification(what: int) -> void:
	if what != NOTIFICATION_SORT_CHILDREN:
		return

	var gap := get_theme_constant(&"h_separation")
	var lead := get_theme_constant(&"v_separation")
	var y := 0.0

	for pair in _pairs():
		var name_cell: Control = pair[0]
		var value_cell: Control = pair[1]
		var name_size := name_cell.get_combined_minimum_size()
		var value_size := value_cell.get_combined_minimum_size()
		var height := maxf(name_size.y, value_size.y)

		var slack := maxf(size.x - gap - _text_width(name_cell), value_size.x)
		var wish = value_cell.get_meta(&"wish", value_cell.get_combined_minimum_size().x)
		var value_width := minf(clampf(wish, value_size.x, slack), size.x)

		fit_child_in_rect(name_cell,
			Rect2(0, y, maxf(size.x - gap - value_width, 0.0), height))
		fit_child_in_rect(value_cell,
			Rect2(size.x - value_width, y, value_width, height))

		y += height + lead

	queue_redraw()

func _draw() -> void:
	var color := get_theme_color(&"leader")
	var gap := get_theme_constant(&"h_separation")

	for pair in _pairs():
		var name_cell: Control = pair[0]
		var value_cell: Control = pair[1]

		var at := ceilf((_text_width(name_cell) + gap) / STEP) * STEP
		var until := value_cell.position.x - gap
		var y := floorf(name_cell.position.y + name_cell.size.y * 0.5)

		while at + DOT <= until:
			draw_rect(Rect2(at, y, DOT, DOT), color)
			at += STEP

## The visible rows, each a name and the value it names.
func _pairs() -> Array:
	var pairs := []
	var i := 0

	while i + 1 < get_child_count():
		var name_cell := get_child(i) as Control
		var value_cell := get_child(i + 1) as Control
		i += 2

		if name_cell == null or value_cell == null:
			continue

		if name_cell.visible and value_cell.visible:
			pairs.append([name_cell, value_cell])

	return pairs

func _text_width(cell: Control) -> float:
	var label := cell as Label

	if label == null:
		return cell.get_minimum_size().x

	return label.get_theme_font(&"font").get_string_size(label.text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, label.get_theme_font_size(&"font_size")).x
