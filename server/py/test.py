import yummy

requests = []

def pre_deviceid_auth(model: yummy.DeviceIdAuth):
    yummy.set_user_meta("", "")
    pass

def post_deviceid_auth(model: yummy.DeviceIdAuth, success: bool):
    pass

def pre_email_auth(model: yummy.EmailAuth):
    pass

def post_email_auth(model: yummy.EmailAuth, success: bool):
    pass
