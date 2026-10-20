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

    nudge(dx, dy) {
        return new Vec2(this.x + dx, this.y + dy)
    }
}

export const vec2 = (x, y) => new Vec2(x, y)

export function xy(pontos) {
    return { x: pontos.map((p) => p.x), y: pontos.map((p) => p.y) }
}

export function linha2d(pontos, extra) {
    return {
        ...xy(pontos),
        type: 'scatter',
        mode: 'lines',
        hoverinfo: 'skip',
        showlegend: false,
        ...extra,
    }
}

export function pontos2d(pontos, cor, tamanho = 9, extra) {
    return {
        ...xy(pontos),
        type: 'scatter',
        mode: 'markers',
        marker: {
            size: tamanho,
            color: cor,
            ...extra
        },
        hoverinfo: 'skip',
        showlegend: false,
    }
}

export function rotulos2d(rotulos, cor, tamanho = 13) {
    return {
        ...xy(rotulos.map((r) => r.position)),
        text: rotulos.map((r) => r.text),
        textposition: rotulos.map((r) => r.textposition ?? 'middle center'),
        type: 'scatter',
        mode: 'text',
        textfont: {
            color: cor,
            size: tamanho
        },
        hoverinfo: 'skip',
        showlegend: false,
    }
}

export function seta2d(de, para, cor) {
    return {
        x: para.x,
        y: para.y,
        ax: de.x,
        ay: de.y,
        xref: 'x',
        yref: 'y',
        axref: 'x',
        ayref: 'y',
        showarrow: true,
        arrowhead: 2,
        arrowsize: 1.1,
        arrowwidth: 2,
        arrowcolor: cor,
        text: '',
    }
}

export function arco2d(centro, raio, de, ate, passos = 48) {
    return Array.from({ length: passos + 1 }, (_, i) => {
        const angulo = de + (ate - de) * (i / passos)

        return centro.add(vec2(Math.cos(angulo), Math.sin(angulo)).mul(raio))
    })
}

export function cena2d([x0, x1], [y0, y1], extra) {
    const eixo = (faixa) => ({
        visible: false,
        showgrid: false,
        zeroline: false,
        fixedrange: true,
        range: faixa,
        constrain: 'domain',
    })

    return {
        xaxis: eixo([x0, x1]),
        yaxis: {
            ...eixo([y0, y1]),
            scaleanchor: 'x'
        },
        margin: {
            t: 8, r: 8, b: 8, l: 8
        },
        hovermode: false,
        ...extra,
    }
}

const CORES = [
    ['#fc436f', '#f83f6c'],
    ['#009850', '#00944e'],
    ['#0e89ff', '#2b90ff'],
]

export function palette(escuro) {
    return {
        serie: CORES.map((s) => s[escuro ? 1 : 0]),
        tinta: escuro ? '#d5dee8' : '#26292c',
        fraca: escuro ? 'rgba(213,222,232,0.45)' : 'rgba(38,41,44,0.45)',
        preenche: escuro ? 'rgba(213,222,232,0.06)' : 'rgba(38,41,44,0.06)',
    }
}

export function alfa(cor, a) {
    const n = parseInt(cor.slice(1), 16)

    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${a})`
}

export class Vec3 {
    constructor(x, y, z) {
        this.x = x
        this.y = y
        this.z = z
    }

    add(o) {
        return new Vec3(this.x + o.x, this.y + o.y, this.z + o.z)
    }

    sub(o) {
        return new Vec3(this.x - o.x, this.y - o.y, this.z - o.z)
    }

    mul(s) {
        return new Vec3(this.x * s, this.y * s, this.z * s)
    }

    dot(o) {
        return this.x * o.x + this.y * o.y + this.z * o.z
    }

    cross(o) {
        return new Vec3(
            this.y * o.z - this.z * o.y,
            this.z * o.x - this.x * o.z,
            this.x * o.y - this.y * o.x,
        )
    }

    length() {
        return Math.sqrt(this.dot(this))
    }

    normalized() {
        return this.mul(1 / this.length())
    }
}

export const vec3 = (x, y, z) => new Vec3(x, y, z)

// No Plotly o eixo vertical é o z, então precisa girar -90° em torno
// do eixo x.
export function xyz(pontos) {
    return {
        x: pontos.map((p) => p && p.x),
        y: pontos.map((p) => p && -p.z),
        z: pontos.map((p) => p && p.y),
    }
}

export function linha3d(pontos, extra) {
    return {
        ...xyz(pontos),
        type: 'scatter3d',
        mode: 'lines',
        hoverinfo: 'skip',
        showlegend: false,
        ...extra,
    }
}

export function malha3d(pontos, faces, extra) {
    return {
        ...xyz(pontos),
        ...faces,
        type: 'mesh3d',
        hoverinfo: 'skip',
        showlegend: false,
        ...extra,
    }
}

export function pontos3d(pontos, cor, tamanho = 5) {
    return {
        ...xyz(pontos),
        type: 'scatter3d',
        mode: 'markers',
        marker: {
            size: tamanho,
            color: cor
        },
        hoverinfo: 'skip',
        showlegend: false,
    }
}

export function rotulos3d(rotulos, cor, tamanho = 13) {
    return {
        ...xyz(rotulos.map((r) => r.position)),
        text: rotulos.map((r) => r.text),
        textposition: rotulos.map((r) => r.textposition ?? 'middle center'),
        type: 'scatter3d',
        mode: 'text',
        textfont: {
            color: cor,
            size: tamanho
        },
        hoverinfo: 'skip',
        showlegend: false,
    }
}

export function seta3d(de, para, cor, ponta = 0.18) {
    const direcao = para.sub(de).normalized()
    const { x, y, z } = xyz([direcao])

    return [
        linha3d([de, para.sub(direcao.mul(ponta * 0.6))], { line: { color: cor, width: 4 } }),
        {
            ...xyz([para]),
            u: x,
            v: y,
            w: z,
            type: 'cone',
            anchor: 'tip',
            sizemode: 'absolute',
            sizeref: ponta,
            colorscale: [
                [0, cor],
                [1, cor]
            ],
            showscale: false,
            lighting: {
                ambient: 1
            },
            hoverinfo: 'skip',
            showlegend: false,
        },
    ]
}

export function arco3d(centro, raio, u, v, de = 0, ate = 2 * Math.PI, passos = 64) {
    return Array.from({ length: passos + 1 }, (_, i) => {
        const angulo = de + (ate - de) * (i / passos)

        return centro.add(u.mul(Math.cos(angulo) * raio)).add(v.mul(Math.sin(angulo) * raio))
    })
}

export function paralelepipedo(origem, a, b, c) {
    const base = [origem, origem.add(a), origem.add(a).add(b), origem.add(b)]
    const topo = base.map((p) => p.add(c))

    return {
        cantos: [
            ...base,
            ...topo,
        ],
        faces: {
            i: [0, 0, 4, 4, 0, 0, 1, 1, 2, 2, 3, 3],
            j: [1, 2, 5, 6, 1, 5, 2, 6, 3, 7, 0, 4],
            k: [2, 3, 6, 7, 5, 4, 6, 5, 7, 6, 4, 7],
        },
        arestas: [
            ...base, base[0], null,
            ...topo, topo[0], null,
            ...base.flatMap((p, n) => [p, topo[n], null]),
        ],
    }
}

export function cena3d(olho, alcance, zoom = 1.45) {
    const eixo = (faixa) => ({
        visible: false,
        showgrid: false,
        zeroline: false,
        showspikes: false,
        range: faixa,
    })

    const { x, y, z } = xyz([olho.normalized().mul(2.165)])

    const largura = {
        x: alcance.x[1] - alcance.x[0],
        y: alcance.z[1] - alcance.z[0],
        z: alcance.y[1] - alcance.y[0],
    }

    const maior = Math.max(largura.x, largura.y, largura.z)

    return {
        scene: {
            xaxis: eixo(alcance.x),
            yaxis: eixo([-alcance.z[1], -alcance.z[0]]),
            zaxis: eixo(alcance.y),
            aspectmode: 'manual',
            aspectratio: {
                x: largura.x / maior * zoom,
                y: largura.y / maior * zoom,
                z: largura.z / maior * zoom,
            },
            camera: {
                eye: {
                    x: x[0], y: y[0], z: z[0]
                },
                up: {
                    x: 0, y: 0, z: 1
                },
                projection: {
                    type: 'orthographic'
                },
            },
            uirevision: 'fixa',
        },
    }
}
