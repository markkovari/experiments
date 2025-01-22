FROM node:20

COPY package*.json .

RUN npm ci

COPY . .

CMD ["node index.js"]