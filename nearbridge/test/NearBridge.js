const NearBridge = artifacts.require('NearBridge');

contract('NearBridge', function ([_, addr1]) {
    describe('NearBridge', async function () {
        it('should be ok', async function () {
            this.token = await NearBridge.new();
        });
    });
});
