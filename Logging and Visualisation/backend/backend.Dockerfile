FROM node:20
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY backend.js .
EXPOSE 3000
CMD ["node", "backend.js"]
