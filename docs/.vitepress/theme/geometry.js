export class Vec2 {
    constructor(x, y) {
        this.x = x
        this.y = y
    }

    add(o) {
        return new Vec2(this.x + o.x, this.y + o.y)
    }

    sub(o) {
        return new Vec2(this.x - o.x, this.y - o.y)
    }

    mul(s) {
        return new Vec2(this.x * s, this.y * s)
    }

    dot(o) {
        return this.x * o.x + this.y * o.y
    }

    cross(o) {
        return this.x * o.y - this.y * o.x
    }

    length() {
        return Math.sqrt(this.dot(this))
    }

    normalized() {
        return this.mul(1 / this.length())
    }

    rotated(angulo) {
        const sen = Math.sin(angulo), cos = Math.cos(angulo)

        return new Vec2(this.x * cos - this.y * sen, this.x * sen + this.y * cos)
    }

    label(from, nudge = 0.42) {
        const perpendicular = vec2(-this.y, this.x).normalized()

        return from.add(this.mul(0.5).add(perpendicular.mul(nudge)))
    }

    nudged_up(nudge) {
        return new Vec2(this.x, this.y + nudge)
    }

    nudged_right(nudge) {
        return new Vec2(this.x + nudge, this.y)
    }
}

export const vec2 = (x, y) => new Vec2(x, y)

export function xy(pontos) {
    return { x: pontos.map((p) => p.x), y: pontos.map((p) => p.y) }
}

const CORES = [
    ['#2a78d6', '#3987e5'],
    ['#eb6834', '#d95926'],
    ['#1baf7a', '#199e70'],
]

export function palette(escuro) {
    return {
        serie: CORES.map((s) => s[escuro ? 1 : 0]),
        tinta: escuro ? '#dfdfd6' : '#3c3c43',
        fraca: escuro ? 'rgba(223,223,214,0.45)' : 'rgba(60,60,67,0.45)',
        preenche: escuro ? 'rgba(223,223,214,0.06)' : 'rgba(60,60,67,0.06)',
        fundo: escuro ? '#1b1b1f' : '#ffffff',
    }
}

export function eixosIguais([x0, x1], [y0, y1]) {
    return {
        xaxis: {
            visible: false,
            showgrid: false,
            zeroline: false,
            fixedrange: true,
            range: [x0, x1],
            constrain: 'domain'
        },
        yaxis: {
            visible: false,
            showgrid: false,
            zeroline: false,
            fixedrange: true,
            range: [y0, y1],
            constrain: 'domain',
            scaleanchor: 'x'
        },
    }
}

export function seta(de, para, cor) {
    return {
        x: para.x, y: para.y,
        ax: de.x, ay: de.y,
        xref: 'x', yref: 'y', axref: 'x', ayref: 'y',
        showarrow: true, arrowhead: 2, arrowsize: 1.1, arrowwidth: 2, arrowcolor: cor,
        text: '',
    }
}

export function alfa(cor, a) {
    const n = parseInt(cor.slice(1), 16)

    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${a})`
}

export function arco(centro, raio, de, ate, passos = 48) {
    return Array.from({ length: passos + 1 }, (_, i) => {
        const angulo = de + (ate - de) * (i / passos)

        return centro.add(vec2(Math.cos(angulo), Math.sin(angulo)).mul(raio))
    })
}
