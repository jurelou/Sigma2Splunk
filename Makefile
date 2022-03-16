release:
	docker build -t sigma2splunk .
	docker image save sigma2splunk > sigma2splunk.tgz
