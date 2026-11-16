import { defineConfig } from 'vitepress'

export default defineConfig({
  base: process.env.GITHUB_ACTIONS ? '/tcc2/' : '/',

  lang: 'pt-BR',
  title: 'TCC2',
  description: 'Rasterização e ray tracing em Rust, com a teoria de cada estágio',

  markdown: { lineNumbers: true },

  themeConfig: {
    nav: [
      { text: 'Rasterização', link: '/rasterizacao/' },
      { text: 'Ray tracing', link: '/raytracing/' },
      { text: 'Código', link: 'https://github.com/mais-um-vini-vanelli/tcc2' }
    ],

    sidebar: [
      {
        text: 'Rasterização',
        collapsed: false,
        items: [
          { text: 'A pipeline', link: '/rasterizacao/' },
          { text: 'Tesselação', link: '/rasterizacao/tesselacao' },
          { text: 'Frustum e recorte', link: '/rasterizacao/frustum-e-recorte' },
          { text: 'Projeção', link: '/rasterizacao/projecao' },
          { text: 'O triângulo', link: '/rasterizacao/triangulo' }
        ]
      },
      {
        text: 'Ray tracing',
        collapsed: false,
        items: [
          { text: 'A pipeline', link: '/raytracing/' },
          { text: 'Geração de raios', link: '/raytracing/geracao-de-raios' },
          { text: 'Interseções', link: '/raytracing/intersecoes' },
          { text: 'Whitted', link: '/raytracing/whitted' },
          { text: 'Reflexão e refração', link: '/raytracing/reflexao-e-refracao' },
          { text: 'BVH', link: '/raytracing/bvh' }
        ]
      },
      {
        text: 'Fundamentos',
        collapsed: false,
        items: [
          { text: 'Câmera', link: '/fundamentos/camera' },
          { text: 'Transformações', link: '/fundamentos/transform' },
          { text: 'Texturas e mipmaps', link: '/fundamentos/texturas' },
          { text: 'Tangentes e normal mapping', link: '/fundamentos/tangentes' },
          { text: 'Shading', link: '/fundamentos/shading' },
          { text: 'Cor e saída', link: '/fundamentos/cor-e-saida' },
        ]
      },
      {
        text: 'Apêndice',
        collapsed: true,
        items: [
          { text: 'Regra de Cramer', link: '/apendice/cramer' },
          { text: 'Produto escalar e vetorial', link: '/apendice/vetores' },
          { text: 'Ponto mais próximo entre retas', link: '/apendice/reta-mais-proxima' }
        ]
      }
    ],

    outline: { level: [2, 3], label: 'Nesta página' },
    docFooter: { prev: 'Anterior', next: 'Próxima' },
    lightModeSwitchTitle: 'Tema claro',
    darkModeSwitchTitle: 'Tema escuro',
    returnToTopLabel: 'Voltar ao topo',
    search: {
      provider: 'local',
      options: {
        translations: {
          button: { buttonText: 'Buscar', buttonAriaLabel: 'Buscar' },
          modal: {
            displayDetails: 'Mostrar detalhes',
            resetButtonTitle: 'Limpar busca',
            noResultsText: 'Nenhum resultado para',
            footer: {
              selectText: 'selecionar',
              navigateText: 'navegar',
              closeText: 'fechar'
            }
          }
        }
      }
    }
  }
})
