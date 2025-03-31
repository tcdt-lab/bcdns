FROM node:20-slim

WORKDIR /usr/src/app

COPY package*.json ./

RUN npm install

COPY . .

# Expose port for web mode
EXPOSE 3000

# Environment variables for operation modes
ENV BCDNS_MODE=web \
    BCDNS_WEB_PORT=3000

CMD ["npm", "start"]
