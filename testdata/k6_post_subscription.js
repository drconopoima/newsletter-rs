import http from 'k6/http';
const SERVER_ADDR = __ENV.SERVER_ADDR || 'localhost:8000';
export default function () {
  let randomnumber = String(Math.floor(Math.random() * (10000000000 - 1)));
  let data = `email=${randomnumber}%40drconopoima.com&name=${randomnumber}`;
  http.post('http://' + SERVER_ADDR + '/subscription', data, {     
    tags: { name: 'POST /subscription?name=[id]&email=[id]@drconopoima.com' },
    headers: { 'Content-Type': 'application/x-www-form-urlencoded'}
  });
}
