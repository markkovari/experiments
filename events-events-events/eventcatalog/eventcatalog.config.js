export default {
  cId: 'e3d7a06e-fefe-446d-ad8e-73a908d8b64d',
  title: 'Event Driven Architecture',
  tagline: 'Discover and document our event-driven ecosystem',
  organizationName: 'Our Org',
  projectName: 'EDA Portal',
  editUrl: 'https://github.com/markkovari/events-events-events',
  trailingSlash: false,
  primaryRGB: '71, 160, 227',
  logo: {
    alt: 'EventCatalog Logo',
    src: 'logo.svg',
  },
  generators: [
    [
      '@eventcatalog/plugin-doc-generator-asyncapi',
      {
        pathToSpec: [
          '../api/order-service.yaml',
          '../api/payment-service.yaml',
          '../api/shipping-service.yaml'
        ],
      },
    ],
  ],
};
