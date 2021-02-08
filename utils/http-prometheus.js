const http = require('http')

class HttpPrometheus {
  constructor (port, metricsPrefix = 'near_bridge_') {
    const self = this
    self.metricsPrefix = metricsPrefix
    self.promClient = require('prom-client')
    self.promClient.collectDefaultMetrics({
      register: new self.promClient.Registry()
    })

    if (port !== 'null') {
      self.port = port
      // create a server object:
      self.server = http.createServer(async function (req, res) {
        if (req.url === '/metrics') {
          const metrics = await self.promClient.register.metrics()
          res.write(metrics) // write a response to the client
          res.end() // end the response
          return
        }
        res.write('Not Found')
        res.end()
      })

      self.server.listen(self.port)
    }
  }

  gauge (name, help, labels = {}) {
    const self = this
    const gauge = new self.promClient.Gauge({
      name: self.metricsPrefix + name,
      help,
      labelNames: Object.keys(labels)
    })
    self.promClient.register.registerMetric(gauge)
    return gauge
  }

  counter (name, help, labels = {}) {
    const self = this
    const counter = new self.promClient.Counter({
      name: self.metricsPrefix + name,
      help,
      labelNames: Object.keys(labels)
    })
    self.promClient.register.registerMetric(counter)
    return counter
  }
}

module.exports = {
  HttpPrometheus
}
