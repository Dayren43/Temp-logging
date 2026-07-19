FROM node:20-slim
WORKDIR /app
COPY package*.json ./
# npm ci is reproducible (honors the lockfile); --omit=dev skips devDependencies
RUN npm ci --omit=dev
COPY backend.js .
ENV NODE_ENV=production
USER node
EXPOSE 3000
# TCP liveness check via node itself (slim has no curl/wget/nc)
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
    CMD node -e "require('net').connect(3000,'127.0.0.1').on('connect',()=>process.exit(0)).on('error',()=>process.exit(1))"
CMD ["node", "backend.js"]
