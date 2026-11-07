import http from 'k6/http';

const SERVER_ADDR = __ENV.SERVER_ADDR || 'localhost:65080';

export default function () {
  http.get('http://' + SERVER_ADDR + '/healthcheck');
}
